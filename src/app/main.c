#include <u8g2.h>
#include <stdint.h>

// Import native functions
#define WASM_IMPORT(name) __attribute__((import_module("env"), import_name(name)))

WASM_IMPORT("host_lcd_update") extern void host_lcd_update(uint8_t* buffer, int size);
WASM_IMPORT("host_get_input") extern uint32_t host_get_input();
WASM_IMPORT("host_delay") extern void host_delay(int ms);

// Global U8g2 instance
u8g2_t u8g2;

// Display buffer (1024 bytes for 128x64 monochrome)
// u8g2 usually allocates this internally or provided. 
// We will use a full buffer mode.
// 128x64 = 8192 bits = 1024 bytes.
// Page buffer is smaller, but let's try full buffer for simplicity in "on_tick".

// Setup callback for u8g2 to write to our buffer? 
// Or just let u8g2 manage it and we access it?
// We will use u8g2_Setup_ssd1306_128x64_noname_f (full buffer)

uint8_t u8x8_gpio_and_delay_template(u8x8_t *u8x8, uint8_t msg, uint8_t arg_int, void *arg_ptr) {
    return 1;
}

uint8_t u8x8_byte_template(u8x8_t *u8x8, uint8_t msg, uint8_t arg_int, void *arg_ptr) {
    return 1;
}

void on_tick() {
    static int initialized = 0;
    static int x = 10;
    static int y = 10;

    if (!initialized) {
        // Init u8g2
        // We use a "dummy" callback because we don't effectively write to I2C/SPI, 
        // we just want the buffer.
        u8g2_Setup_ssd1306_128x64_noname_f(&u8g2, U8G2_R0, u8x8_byte_template, u8x8_gpio_and_delay_template);
        u8g2_InitDisplay(&u8g2);
        u8g2_SetPowerSave(&u8g2, 0);
        initialized = 1;
    }

    // Input
    uint32_t input = host_get_input();
    if (input & (1 << 0)) y--; // Up
    if (input & (1 << 1)) y++; // Down
    if (input & (1 << 2)) x--; // Left
    if (input & (1 << 3)) x++; // Right

    u8g2_ClearBuffer(&u8g2);
    u8g2_DrawCircle(&u8g2, x, y, 10, U8G2_DRAW_ALL);
    u8g2_SendBuffer(&u8g2); // This usually sends to I2C/SPI.
    
    // But we want to send the INTERNAL buffer to host.
    // Access u8g2 private buffer?
    // u8g2.tile_buf_ptr might point to it? Or we can use `u8g2_GetBufferPtr(&u8g2)`.
    
    uint8_t* buf = u8g2_GetBufferPtr(&u8g2);
    // Size for 128x64 1 bit is 1024 bytes.
    host_lcd_update(buf, 1024);
}

int main() {
    return 0;
}
