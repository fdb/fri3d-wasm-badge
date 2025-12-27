#pragma once

#include <u8g2.h>
#include <cstdint>

namespace fri3d {

// Color values matching the SDK
enum class Color : uint8_t {
    White = 0,
    Black = 1,
    XOR = 2
};

// Font values matching the SDK
enum class Font : uint8_t {
    Primary = 0,
    Secondary = 1,
    Keyboard = 2,
    BigNumbers = 3
};

/**
 * Canvas provides drawing operations backed by u8g2.
 * This is used by the runtime to implement native functions for WASM apps.
 */
class Canvas {
public:
    /**
     * Initialize the canvas with a u8g2 instance.
     * The u8g2 instance is owned by the Display.
     */
    void init(u8g2_t* u8g2);

    /**
     * Get the u8g2 instance (for direct access if needed).
     */
    u8g2_t* getU8g2() { return m_u8g2; }

    // Screen dimensions
    uint32_t width() const;
    uint32_t height() const;

    // Configuration
    void clear();
    void setColor(Color color);
    void setFont(Font font);
    Color getColor() const { return m_currentColor; }

    // Basic drawing
    void drawDot(int32_t x, int32_t y);
    void drawLine(int32_t x1, int32_t y1, int32_t x2, int32_t y2);

    // Rectangles
    void drawFrame(int32_t x, int32_t y, uint32_t w, uint32_t h);
    void drawBox(int32_t x, int32_t y, uint32_t w, uint32_t h);
    void drawRFrame(int32_t x, int32_t y, uint32_t w, uint32_t h, uint32_t r);
    void drawRBox(int32_t x, int32_t y, uint32_t w, uint32_t h, uint32_t r);

    // Circles (with XOR-safe implementations)
    void drawCircle(int32_t x, int32_t y, uint32_t r);
    void drawDisc(int32_t x, int32_t y, uint32_t r);

    // Text
    void drawStr(int32_t x, int32_t y, const char* str);
    uint32_t stringWidth(const char* str);

private:
    u8g2_t* m_u8g2 = nullptr;
    Color m_currentColor = Color::Black;

    // XOR-safe circle drawing helpers
    void drawXorCircle(int32_t x0, int32_t y0, uint32_t r);
    void drawXorDisc(int32_t x0, int32_t y0, uint32_t r);
};

} // namespace fri3d
