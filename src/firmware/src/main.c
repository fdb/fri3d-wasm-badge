/**
 * Fri3d Badge Firmware
 * ESP32-S3 main entry point
 */

#include <stdio.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_log.h"
#include "esp_system.h"

#include "display_spi.h"
#include "input_gpio.h"

static const char* log_tag = "fri3d";

void app_main(void) {
    ESP_LOGI(log_tag, "Fri3d Badge starting...");

    ESP_LOGI(log_tag, "Initializing display...");
    if (!display_init()) {
        ESP_LOGE(log_tag, "Failed to initialize display");
        return;
    }

    ESP_LOGI(log_tag, "Initializing input...");
    input_init();

    ESP_LOGI(log_tag, "Fri3d Badge ready!");

    while (true) {
        input_poll();
        vTaskDelay(pdMS_TO_TICKS(16));
    }
}
