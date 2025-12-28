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
// #include "storage_sd.h"  // TODO: Implement SD card support

// Runtime components (from shared runtime)
// Note: These need to be adapted for ESP-IDF build
// For now, this is a skeleton that shows the intended structure

static const char* TAG = "fri3d";

void app_main(void) {
    ESP_LOGI(TAG, "Fri3d Badge starting...");

    // Initialize display
    ESP_LOGI(TAG, "Initializing display...");
    if (!display_init()) {
        ESP_LOGE(TAG, "Failed to initialize display");
        return;
    }

    // Initialize input
    ESP_LOGI(TAG, "Initializing input...");
    input_init();

    // Initialize storage (SD card or SPIFFS)
    // ESP_LOGI(TAG, "Initializing storage...");
    // storage_init();

    // TODO: Initialize WAMR runtime
    // TODO: Initialize app manager with launcher

    ESP_LOGI(TAG, "Fri3d Badge ready!");

    // Main loop
    while (true) {
        // Poll input
        input_poll();

        // TODO: Update input manager (short/long press detection)
        // TODO: Handle reset combo

        // TODO: Render current app/launcher
        // TODO: Flush display

        // Delay for ~60 FPS
        vTaskDelay(pdMS_TO_TICKS(16));
    }
}
