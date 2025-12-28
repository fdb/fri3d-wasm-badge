/**
 * GPIO Button Input handler for ESP32-S3
 */

#include "input_gpio.h"
#include "driver/gpio.h"
#include "esp_timer.h"
#include "esp_log.h"

static const char* TAG = "input";

// GPIO Pin Configuration (invented - adjust for actual hardware)
#define PIN_BTN_UP      GPIO_NUM_9
#define PIN_BTN_DOWN    GPIO_NUM_10
#define PIN_BTN_LEFT    GPIO_NUM_11
#define PIN_BTN_RIGHT   GPIO_NUM_12
#define PIN_BTN_OK      GPIO_NUM_13
#define PIN_BTN_BACK    GPIO_NUM_14

// Debounce time in milliseconds
#define DEBOUNCE_MS     20

// Button state
typedef struct {
    gpio_num_t pin;
    bool pressed;
    bool last_state;
    uint32_t last_change_ms;
} button_state_t;

static button_state_t g_buttons[INPUT_KEY_COUNT] = {
    { PIN_BTN_UP,    false, false, 0 },
    { PIN_BTN_DOWN,  false, false, 0 },
    { PIN_BTN_LEFT,  false, false, 0 },
    { PIN_BTN_RIGHT, false, false, 0 },
    { PIN_BTN_OK,    false, false, 0 },
    { PIN_BTN_BACK,  false, false, 0 },
};

// Event queue (simple ring buffer)
#define EVENT_QUEUE_SIZE 16
static input_event_t g_event_queue[EVENT_QUEUE_SIZE];
static size_t g_queue_head = 0;
static size_t g_queue_tail = 0;

static void queue_event(input_key_t key, input_type_t type) {
    size_t next_head = (g_queue_head + 1) % EVENT_QUEUE_SIZE;
    if (next_head != g_queue_tail) {  // Not full
        g_event_queue[g_queue_head].key = key;
        g_event_queue[g_queue_head].type = type;
        g_queue_head = next_head;
    }
}

void input_init(void) {
    ESP_LOGI(TAG, "Initializing GPIO input...");

    // Configure all button pins as input with pull-up
    // Buttons are expected to be active-low (pressed = GND)
    gpio_config_t io_conf = {0};
    io_conf.intr_type = GPIO_INTR_DISABLE;
    io_conf.mode = GPIO_MODE_INPUT;
    io_conf.pull_up_en = GPIO_PULLUP_ENABLE;
    io_conf.pull_down_en = GPIO_PULLDOWN_DISABLE;

    for (int i = 0; i < INPUT_KEY_COUNT; i++) {
        io_conf.pin_bit_mask = (1ULL << g_buttons[i].pin);
        gpio_config(&io_conf);
    }

    ESP_LOGI(TAG, "GPIO input initialized");
}

void input_poll(void) {
    uint32_t now_ms = input_get_time_ms();

    for (int i = 0; i < INPUT_KEY_COUNT; i++) {
        button_state_t* btn = &g_buttons[i];

        // Read button state (active low)
        bool current = (gpio_get_level(btn->pin) == 0);

        // Debounce
        if (current != btn->last_state) {
            btn->last_state = current;
            btn->last_change_ms = now_ms;
        }

        // Check if debounce period has passed
        if ((now_ms - btn->last_change_ms) >= DEBOUNCE_MS) {
            if (current != btn->pressed) {
                btn->pressed = current;

                // Queue event
                queue_event(
                    (input_key_t)i,
                    current ? INPUT_TYPE_PRESS : INPUT_TYPE_RELEASE
                );
            }
        }
    }
}

bool input_has_event(void) {
    return g_queue_head != g_queue_tail;
}

input_event_t input_get_event(void) {
    input_event_t event = g_event_queue[g_queue_tail];
    g_queue_tail = (g_queue_tail + 1) % EVENT_QUEUE_SIZE;
    return event;
}

bool input_is_pressed(input_key_t key) {
    if (key >= INPUT_KEY_COUNT) return false;
    return g_buttons[key].pressed;
}

uint32_t input_get_time_ms(void) {
    return (uint32_t)(esp_timer_get_time() / 1000);
}
