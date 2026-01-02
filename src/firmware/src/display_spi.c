/**
 * SPI Display driver for ESP32-S3
 * Uses u8g2 with SSD1306 128x64 display
 */

#include "display_spi.h"
#include "driver/spi_master.h"
#include "driver/gpio.h"
#include "esp_log.h"
#include <string.h>

static const char* log_tag = "display";

#define PIN_MOSI    GPIO_NUM_6
#define PIN_MISO    GPIO_NUM_8
#define PIN_SCK     GPIO_NUM_7
#define PIN_CS      GPIO_NUM_5
#define PIN_DC      GPIO_NUM_4
#define PIN_RST     GPIO_NUM_48

#define SPI_HOST    SPI2_HOST
#define SPI_FREQ_HZ (10 * 1000 * 1000)

#define SCREEN_WIDTH  128
#define SCREEN_HEIGHT 64

static u8g2_t g_u8g2;
static spi_device_handle_t g_spi;

static uint8_t u8x8_byte_esp32_hw_spi(u8x8_t* u8x8, uint8_t msg, uint8_t arg_int, void* arg_ptr) {
    switch (msg) {
        case U8X8_MSG_BYTE_SEND: {
            spi_transaction_t trans;
            memset(&trans, 0, sizeof(trans));
            trans.length = arg_int * 8;
            trans.tx_buffer = arg_ptr;
            spi_device_transmit(g_spi, &trans);
            break;
        }
        case U8X8_MSG_BYTE_INIT:
            break;
        case U8X8_MSG_BYTE_SET_DC:
            gpio_set_level(PIN_DC, arg_int);
            break;
        case U8X8_MSG_BYTE_START_TRANSFER:
            break;
        case U8X8_MSG_BYTE_END_TRANSFER:
            break;
        default:
            return 0;
    }
    return 1;
}

static uint8_t u8x8_gpio_and_delay_esp32(u8x8_t* u8x8, uint8_t msg, uint8_t arg_int, void* arg_ptr) {
    switch (msg) {
        case U8X8_MSG_GPIO_AND_DELAY_INIT:
            break;
        case U8X8_MSG_DELAY_MILLI:
            vTaskDelay(pdMS_TO_TICKS(arg_int));
            break;
        case U8X8_MSG_DELAY_10MICRO:
            ets_delay_us(arg_int * 10);
            break;
        case U8X8_MSG_DELAY_100NANO:
            ets_delay_us(1);
            break;
        case U8X8_MSG_GPIO_CS:
            break;
        case U8X8_MSG_GPIO_DC:
            gpio_set_level(PIN_DC, arg_int);
            break;
        case U8X8_MSG_GPIO_RESET:
            gpio_set_level(PIN_RST, arg_int);
            break;
        default:
            return 0;
    }
    return 1;
}

bool display_init(void) {
    ESP_LOGI(log_tag, "Initializing SPI display...");

    gpio_config_t io_conf = {0};
    io_conf.intr_type = GPIO_INTR_DISABLE;
    io_conf.mode = GPIO_MODE_OUTPUT;
    io_conf.pin_bit_mask = (1ULL << PIN_DC) | (1ULL << PIN_RST);
    io_conf.pull_down_en = GPIO_PULLDOWN_DISABLE;
    io_conf.pull_up_en = GPIO_PULLUP_DISABLE;
    gpio_config(&io_conf);

    spi_bus_config_t bus_cfg = {0};
    bus_cfg.mosi_io_num = PIN_MOSI;
    bus_cfg.miso_io_num = PIN_MISO;
    bus_cfg.sclk_io_num = PIN_SCK;
    bus_cfg.quadwp_io_num = -1;
    bus_cfg.quadhd_io_num = -1;
    bus_cfg.max_transfer_sz = SCREEN_WIDTH * SCREEN_HEIGHT / 8 + 8;

    esp_err_t ret = spi_bus_initialize(SPI_HOST, &bus_cfg, SPI_DMA_CH_AUTO);
    if (ret != ESP_OK) {
        ESP_LOGE(log_tag, "Failed to initialize SPI bus: %s", esp_err_to_name(ret));
        return false;
    }

    spi_device_interface_config_t dev_cfg = {0};
    dev_cfg.clock_speed_hz = SPI_FREQ_HZ;
    dev_cfg.mode = 0;
    dev_cfg.spics_io_num = PIN_CS;
    dev_cfg.queue_size = 1;

    ret = spi_bus_add_device(SPI_HOST, &dev_cfg, &g_spi);
    if (ret != ESP_OK) {
        ESP_LOGE(log_tag, "Failed to add SPI device: %s", esp_err_to_name(ret));
        return false;
    }

    u8g2_Setup_ssd1306_128x64_noname_f(
        &g_u8g2, U8G2_R0,
        u8x8_byte_esp32_hw_spi,
        u8x8_gpio_and_delay_esp32
    );

    u8g2_InitDisplay(&g_u8g2);
    u8g2_SetPowerSave(&g_u8g2, 0);
    u8g2_ClearBuffer(&g_u8g2);

    ESP_LOGI(log_tag, "Display initialized");
    return true;
}

u8g2_t* display_get_u8g2(void) {
    return &g_u8g2;
}

void display_flush(void) {
    u8g2_SendBuffer(&g_u8g2);
}

void display_clear(void) {
    u8g2_ClearBuffer(&g_u8g2);
}
