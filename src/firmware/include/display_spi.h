#pragma once

#include <u8g2.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Initialize the SPI display.
 * @return true on success
 */
bool display_init(void);

/**
 * Get the u8g2 instance for drawing operations.
 */
u8g2_t* display_get_u8g2(void);

/**
 * Flush the display buffer to the screen.
 */
void display_flush(void);

/**
 * Clear the display buffer.
 */
void display_clear(void);

#ifdef __cplusplus
}
#endif
