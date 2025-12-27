// ============================================================================
// Fri3d Badge Web Platform
// ============================================================================
// Pure JavaScript platform implementation - no Emscripten needed!
// Uses browser's native WebAssembly API to run WASM apps.

// ============================================================================
// Bitmap Fonts - 5x7 font (similar to u8g2_font_5x7_tf)
// ============================================================================
// Each character is 5 columns wide, 7 rows tall
// Stored as 5 bytes per character (one byte per column, LSB at top)

const FONT_5X7 = {
    width: 5,
    height: 7,
    baseline: 6,  // Y offset from top to baseline
    spacing: 1,   // Space between characters
    // ASCII 32-126 (printable characters)
    glyphs: {
        32: [0x00, 0x00, 0x00, 0x00, 0x00], // space
        33: [0x00, 0x00, 0x5F, 0x00, 0x00], // !
        34: [0x00, 0x07, 0x00, 0x07, 0x00], // "
        35: [0x14, 0x7F, 0x14, 0x7F, 0x14], // #
        36: [0x24, 0x2A, 0x7F, 0x2A, 0x12], // $
        37: [0x23, 0x13, 0x08, 0x64, 0x62], // %
        38: [0x36, 0x49, 0x55, 0x22, 0x50], // &
        39: [0x00, 0x05, 0x03, 0x00, 0x00], // '
        40: [0x00, 0x1C, 0x22, 0x41, 0x00], // (
        41: [0x00, 0x41, 0x22, 0x1C, 0x00], // )
        42: [0x08, 0x2A, 0x1C, 0x2A, 0x08], // *
        43: [0x08, 0x08, 0x3E, 0x08, 0x08], // +
        44: [0x00, 0x50, 0x30, 0x00, 0x00], // ,
        45: [0x08, 0x08, 0x08, 0x08, 0x08], // -
        46: [0x00, 0x60, 0x60, 0x00, 0x00], // .
        47: [0x20, 0x10, 0x08, 0x04, 0x02], // /
        48: [0x3E, 0x51, 0x49, 0x45, 0x3E], // 0
        49: [0x00, 0x42, 0x7F, 0x40, 0x00], // 1
        50: [0x42, 0x61, 0x51, 0x49, 0x46], // 2
        51: [0x21, 0x41, 0x45, 0x4B, 0x31], // 3
        52: [0x18, 0x14, 0x12, 0x7F, 0x10], // 4
        53: [0x27, 0x45, 0x45, 0x45, 0x39], // 5
        54: [0x3C, 0x4A, 0x49, 0x49, 0x30], // 6
        55: [0x01, 0x71, 0x09, 0x05, 0x03], // 7
        56: [0x36, 0x49, 0x49, 0x49, 0x36], // 8
        57: [0x06, 0x49, 0x49, 0x29, 0x1E], // 9
        58: [0x00, 0x36, 0x36, 0x00, 0x00], // :
        59: [0x00, 0x56, 0x36, 0x00, 0x00], // ;
        60: [0x00, 0x08, 0x14, 0x22, 0x41], // <
        61: [0x14, 0x14, 0x14, 0x14, 0x14], // =
        62: [0x41, 0x22, 0x14, 0x08, 0x00], // >
        63: [0x02, 0x01, 0x51, 0x09, 0x06], // ?
        64: [0x32, 0x49, 0x79, 0x41, 0x3E], // @
        65: [0x7E, 0x11, 0x11, 0x11, 0x7E], // A
        66: [0x7F, 0x49, 0x49, 0x49, 0x36], // B
        67: [0x3E, 0x41, 0x41, 0x41, 0x22], // C
        68: [0x7F, 0x41, 0x41, 0x22, 0x1C], // D
        69: [0x7F, 0x49, 0x49, 0x49, 0x41], // E
        70: [0x7F, 0x09, 0x09, 0x01, 0x01], // F
        71: [0x3E, 0x41, 0x41, 0x51, 0x32], // G
        72: [0x7F, 0x08, 0x08, 0x08, 0x7F], // H
        73: [0x00, 0x41, 0x7F, 0x41, 0x00], // I
        74: [0x20, 0x40, 0x41, 0x3F, 0x01], // J
        75: [0x7F, 0x08, 0x14, 0x22, 0x41], // K
        76: [0x7F, 0x40, 0x40, 0x40, 0x40], // L
        77: [0x7F, 0x02, 0x04, 0x02, 0x7F], // M
        78: [0x7F, 0x04, 0x08, 0x10, 0x7F], // N
        79: [0x3E, 0x41, 0x41, 0x41, 0x3E], // O
        80: [0x7F, 0x09, 0x09, 0x09, 0x06], // P
        81: [0x3E, 0x41, 0x51, 0x21, 0x5E], // Q
        82: [0x7F, 0x09, 0x19, 0x29, 0x46], // R
        83: [0x46, 0x49, 0x49, 0x49, 0x31], // S
        84: [0x01, 0x01, 0x7F, 0x01, 0x01], // T
        85: [0x3F, 0x40, 0x40, 0x40, 0x3F], // U
        86: [0x1F, 0x20, 0x40, 0x20, 0x1F], // V
        87: [0x7F, 0x20, 0x18, 0x20, 0x7F], // W
        88: [0x63, 0x14, 0x08, 0x14, 0x63], // X
        89: [0x03, 0x04, 0x78, 0x04, 0x03], // Y
        90: [0x61, 0x51, 0x49, 0x45, 0x43], // Z
        91: [0x00, 0x00, 0x7F, 0x41, 0x41], // [
        92: [0x02, 0x04, 0x08, 0x10, 0x20], // \
        93: [0x41, 0x41, 0x7F, 0x00, 0x00], // ]
        94: [0x04, 0x02, 0x01, 0x02, 0x04], // ^
        95: [0x40, 0x40, 0x40, 0x40, 0x40], // _
        96: [0x00, 0x01, 0x02, 0x04, 0x00], // `
        97: [0x20, 0x54, 0x54, 0x54, 0x78], // a
        98: [0x7F, 0x48, 0x44, 0x44, 0x38], // b
        99: [0x38, 0x44, 0x44, 0x44, 0x20], // c
        100: [0x38, 0x44, 0x44, 0x48, 0x7F], // d
        101: [0x38, 0x54, 0x54, 0x54, 0x18], // e
        102: [0x08, 0x7E, 0x09, 0x01, 0x02], // f
        103: [0x08, 0x14, 0x54, 0x54, 0x3C], // g
        104: [0x7F, 0x08, 0x04, 0x04, 0x78], // h
        105: [0x00, 0x44, 0x7D, 0x40, 0x00], // i
        106: [0x20, 0x40, 0x44, 0x3D, 0x00], // j
        107: [0x00, 0x7F, 0x10, 0x28, 0x44], // k
        108: [0x00, 0x41, 0x7F, 0x40, 0x00], // l
        109: [0x7C, 0x04, 0x18, 0x04, 0x78], // m
        110: [0x7C, 0x08, 0x04, 0x04, 0x78], // n
        111: [0x38, 0x44, 0x44, 0x44, 0x38], // o
        112: [0x7C, 0x14, 0x14, 0x14, 0x08], // p
        113: [0x08, 0x14, 0x14, 0x18, 0x7C], // q
        114: [0x7C, 0x08, 0x04, 0x04, 0x08], // r
        115: [0x48, 0x54, 0x54, 0x54, 0x20], // s
        116: [0x04, 0x3F, 0x44, 0x40, 0x20], // t
        117: [0x3C, 0x40, 0x40, 0x20, 0x7C], // u
        118: [0x1C, 0x20, 0x40, 0x20, 0x1C], // v
        119: [0x3C, 0x40, 0x30, 0x40, 0x3C], // w
        120: [0x44, 0x28, 0x10, 0x28, 0x44], // x
        121: [0x0C, 0x50, 0x50, 0x50, 0x3C], // y
        122: [0x44, 0x64, 0x54, 0x4C, 0x44], // z
        123: [0x00, 0x08, 0x36, 0x41, 0x00], // {
        124: [0x00, 0x00, 0x7F, 0x00, 0x00], // |
        125: [0x00, 0x41, 0x36, 0x08, 0x00], // }
        126: [0x10, 0x08, 0x08, 0x10, 0x08], // ~
    }
};

// 6x10 font (similar to u8g2_font_6x10_tf) - Primary font
const FONT_6X10 = {
    width: 6,
    height: 10,
    baseline: 8,
    spacing: 1,
    // Use 5x7 glyphs scaled/padded to 6x10
    glyphs: FONT_5X7.glyphs  // Reuse 5x7 glyphs, render with padding
};

// Font configurations matching platform.zig
const FONTS = [
    FONT_6X10,  // 0: primary (6x10)
    FONT_5X7,   // 1: secondary (5x7)
    FONT_5X7,   // 2: keyboard (5x8, using 5x7)
    FONT_6X10,  // 3: big_numbers (10x20, using 6x10 for now)
];

class Fri3dPlatform {
    constructor(canvas) {
        this.canvas = canvas;
        this.ctx = canvas.getContext('2d');
        this.scale = 4; // Display scale (128x64 -> 512x256)

        // Monochrome framebuffer (128x64 = 8192 pixels)
        this.width = 128;
        this.height = 64;
        this.framebuffer = new Uint8Array(this.width * this.height);

        // Current drawing state
        this.color = 1; // 0=black, 1=white, 2=xor (default to white for drawing)
        this.font = 0;

        // Random number generator (simple LCG)
        this.rngState = 42;

        // Current WASM app instance
        this.wasmInstance = null;
        this.wasmMemory = null;

        // Input state
        this.setupInputHandlers();

        // Start render loop
        this.running = true;
        this.lastTime = 0;
        requestAnimationFrame((t) => this.tick(t));
    }

    // ========================================================================
    // App Loading
    // ========================================================================

    async loadApp(url) {
        console.log(`Loading app: ${url}`);

        try {
            const response = await fetch(url);
            const bytes = await response.arrayBuffer();

            // Create import object with platform functions
            const imports = this.getImports();

            // Instantiate the WASM module
            const result = await WebAssembly.instantiate(bytes, imports);
            this.wasmInstance = result.instance;
            this.wasmMemory = this.wasmInstance.exports.memory;

            console.log('App loaded successfully');
            console.log('Exports:', Object.keys(this.wasmInstance.exports));
        } catch (error) {
            console.error('Failed to load app:', error);
        }
    }

    // ========================================================================
    // Platform Imports (provided to WASM)
    // ========================================================================

    getImports() {
        return {
            env: {
                // Canvas functions
                canvas_clear: () => this.canvasClear(),
                canvas_width: () => this.width,
                canvas_height: () => this.height,
                canvas_set_color: (color) => { this.color = color; },
                canvas_set_font: (font) => { this.font = font; },
                canvas_draw_dot: (x, y) => this.drawDot(x, y),
                canvas_draw_line: (x1, y1, x2, y2) => this.drawLine(x1, y1, x2, y2),
                canvas_draw_frame: (x, y, w, h) => this.drawFrame(x, y, w, h),
                canvas_draw_box: (x, y, w, h) => this.drawBox(x, y, w, h),
                canvas_draw_rframe: (x, y, w, h, r) => this.drawRFrame(x, y, w, h, r),
                canvas_draw_rbox: (x, y, w, h, r) => this.drawRBox(x, y, w, h, r),
                canvas_draw_circle: (x, y, r) => this.drawCircle(x, y, r),
                canvas_draw_disc: (x, y, r) => this.drawDisc(x, y, r),
                canvas_draw_str: (x, y, strPtr) => this.drawStr(x, y, strPtr),
                canvas_string_width: (strPtr) => this.stringWidth(strPtr),

                // Random functions
                random_seed: (seed) => { this.rngState = seed; },
                random_get: () => this.randomGet(),
                random_range: (max) => this.randomRange(max),
            }
        };
    }

    // ========================================================================
    // Canvas Implementation
    // ========================================================================

    canvasClear() {
        this.framebuffer.fill(0);
    }

    setPixel(x, y) {
        if (x < 0 || x >= this.width || y < 0 || y >= this.height) return;

        const idx = y * this.width + x;
        if (this.color === 0) { // Black
            this.framebuffer[idx] = 0;
        } else if (this.color === 1) { // White
            this.framebuffer[idx] = 1;
        } else if (this.color === 2) { // XOR
            this.framebuffer[idx] ^= 1;
        }
    }

    drawDot(x, y) {
        this.setPixel(x, y);
    }

    drawLine(x1, y1, x2, y2) {
        // Bresenham's line algorithm
        const dx = Math.abs(x2 - x1);
        const dy = Math.abs(y2 - y1);
        const sx = x1 < x2 ? 1 : -1;
        const sy = y1 < y2 ? 1 : -1;
        let err = dx - dy;

        while (true) {
            this.setPixel(x1, y1);
            if (x1 === x2 && y1 === y2) break;
            const e2 = 2 * err;
            if (e2 > -dy) { err -= dy; x1 += sx; }
            if (e2 < dx) { err += dx; y1 += sy; }
        }
    }

    drawFrame(x, y, w, h) {
        // Top and bottom
        for (let i = 0; i < w; i++) {
            this.setPixel(x + i, y);
            this.setPixel(x + i, y + h - 1);
        }
        // Left and right
        for (let i = 0; i < h; i++) {
            this.setPixel(x, y + i);
            this.setPixel(x + w - 1, y + i);
        }
    }

    drawBox(x, y, w, h) {
        for (let py = y; py < y + h; py++) {
            for (let px = x; px < x + w; px++) {
                this.setPixel(px, py);
            }
        }
    }

    drawRFrame(x, y, w, h, r) {
        if (r === 0 || r > Math.min(w, h) / 2) {
            this.drawFrame(x, y, w, h);
            return;
        }
        // Draw rounded frame
        // Top and bottom (excluding corners)
        for (let i = r; i < w - r; i++) {
            this.setPixel(x + i, y);
            this.setPixel(x + i, y + h - 1);
        }
        // Left and right (excluding corners)
        for (let i = r; i < h - r; i++) {
            this.setPixel(x, y + i);
            this.setPixel(x + w - 1, y + i);
        }
        // Draw corner arcs
        this.drawCornerArc(x + r, y + r, r, 0);           // top-left
        this.drawCornerArc(x + w - 1 - r, y + r, r, 1);   // top-right
        this.drawCornerArc(x + r, y + h - 1 - r, r, 2);   // bottom-left
        this.drawCornerArc(x + w - 1 - r, y + h - 1 - r, r, 3); // bottom-right
    }

    drawCornerArc(cx, cy, r, corner) {
        // Draw quarter arc for rounded corners
        // corner: 0=top-left, 1=top-right, 2=bottom-left, 3=bottom-right
        let px = r;
        let py = 0;
        let err = 0;

        while (px >= py) {
            switch (corner) {
                case 0: // top-left
                    this.setPixel(cx - px, cy - py);
                    this.setPixel(cx - py, cy - px);
                    break;
                case 1: // top-right
                    this.setPixel(cx + px, cy - py);
                    this.setPixel(cx + py, cy - px);
                    break;
                case 2: // bottom-left
                    this.setPixel(cx - px, cy + py);
                    this.setPixel(cx - py, cy + px);
                    break;
                case 3: // bottom-right
                    this.setPixel(cx + px, cy + py);
                    this.setPixel(cx + py, cy + px);
                    break;
            }
            py++;
            if (err <= 0) {
                err += 2 * py + 1;
            }
            if (err > 0) {
                px--;
                err -= 2 * px + 1;
            }
        }
    }

    drawRBox(x, y, w, h, r) {
        if (r === 0 || r > Math.min(w, h) / 2) {
            this.drawBox(x, y, w, h);
            return;
        }
        // Draw center rectangle
        this.drawBox(x, y + r, w, h - 2 * r);
        // Draw top and bottom rectangles (excluding corners)
        this.drawBox(x + r, y, w - 2 * r, r);
        this.drawBox(x + r, y + h - r, w - 2 * r, r);
        // Fill corners
        this.fillCorner(x + r, y + r, r, 0);
        this.fillCorner(x + w - 1 - r, y + r, r, 1);
        this.fillCorner(x + r, y + h - 1 - r, r, 2);
        this.fillCorner(x + w - 1 - r, y + h - 1 - r, r, 3);
    }

    fillCorner(cx, cy, r, corner) {
        // Fill quarter disc for rounded box corners
        for (let dy = 0; dy <= r; dy++) {
            const dx = Math.floor(Math.sqrt(r * r - dy * dy));
            for (let i = 0; i <= dx; i++) {
                switch (corner) {
                    case 0: this.setPixel(cx - i, cy - dy); break;
                    case 1: this.setPixel(cx + i, cy - dy); break;
                    case 2: this.setPixel(cx - i, cy + dy); break;
                    case 3: this.setPixel(cx + i, cy + dy); break;
                }
            }
        }
    }

    drawCircle(x, y, r) {
        // Midpoint circle algorithm
        let px = r;
        let py = 0;
        let err = 0;

        while (px >= py) {
            this.setPixel(x + px, y + py);
            this.setPixel(x + py, y + px);
            this.setPixel(x - py, y + px);
            this.setPixel(x - px, y + py);
            this.setPixel(x - px, y - py);
            this.setPixel(x - py, y - px);
            this.setPixel(x + py, y - px);
            this.setPixel(x + px, y - py);

            py++;
            if (err <= 0) {
                err += 2 * py + 1;
            }
            if (err > 0) {
                px--;
                err -= 2 * px + 1;
            }
        }
    }

    drawDisc(x, y, r) {
        // Filled circle using horizontal lines
        for (let dy = -r; dy <= r; dy++) {
            const dx = Math.floor(Math.sqrt(r * r - dy * dy));
            for (let px = x - dx; px <= x + dx; px++) {
                this.setPixel(px, y + dy);
            }
        }
    }

    // ========================================================================
    // Text Rendering
    // ========================================================================

    drawStr(x, y, strPtr) {
        // Read string from WASM memory
        if (!this.wasmMemory) return;
        const mem = new Uint8Array(this.wasmMemory.buffer);
        let str = '';
        let i = strPtr;
        while (mem[i] !== 0 && i - strPtr < 256) { // Limit string length
            str += String.fromCharCode(mem[i]);
            i++;
        }

        // Get current font
        const fontData = FONTS[this.font] || FONT_5X7;
        const charWidth = fontData.width + fontData.spacing;

        // Draw each character
        let curX = x;
        for (let c = 0; c < str.length; c++) {
            this.drawChar(curX, y, str.charCodeAt(c), fontData);
            curX += charWidth;
        }
    }

    drawChar(x, y, charCode, fontData) {
        // Get glyph data
        const glyph = fontData.glyphs[charCode];
        if (!glyph) {
            // Draw placeholder box for unknown characters
            const oldColor = this.color;
            this.color = 1;
            this.drawFrame(x, y - fontData.baseline + 1, fontData.width, fontData.height - 1);
            this.color = oldColor;
            return;
        }

        // Calculate Y offset (glyphs are drawn from baseline)
        const yOffset = y - fontData.baseline;

        // Draw the glyph (5 columns, 7 rows)
        // Each byte is a column, LSB is at top
        for (let col = 0; col < glyph.length; col++) {
            const colData = glyph[col];
            for (let row = 0; row < 7; row++) {
                if (colData & (1 << row)) {
                    this.setPixel(x + col, yOffset + row);
                }
            }
        }
    }

    stringWidth(strPtr) {
        // Read string length from WASM memory
        if (!this.wasmMemory) return 0;
        const mem = new Uint8Array(this.wasmMemory.buffer);
        let len = 0;
        let i = strPtr;
        while (mem[i] !== 0 && len < 256) {
            len++;
            i++;
        }

        const fontData = FONTS[this.font] || FONT_5X7;
        const charWidth = fontData.width + fontData.spacing;
        return len * charWidth;
    }

    // ========================================================================
    // Random Number Generator
    // ========================================================================

    randomGet() {
        // Simple LCG (same constants as many implementations)
        this.rngState = (this.rngState * 1103515245 + 12345) & 0x7fffffff;
        return this.rngState;
    }

    randomRange(max) {
        if (max === 0) return 0;
        return this.randomGet() % max;
    }

    // ========================================================================
    // Input Handling
    // ========================================================================

    setupInputHandlers() {
        const keyMap = {
            'ArrowUp': 0,    // up
            'ArrowDown': 1,  // down
            'ArrowLeft': 2,  // left
            'ArrowRight': 3, // right
            'Enter': 4,      // ok
            'z': 4,          // ok (alternative)
            'Z': 4,          // ok
            'Backspace': 5,  // back
            'x': 5,          // back (alternative)
            'X': 5,          // back
            'Escape': 5,     // back (alternative)
        };

        document.addEventListener('keydown', (e) => {
            if (keyMap.hasOwnProperty(e.key)) {
                e.preventDefault();
                this.handleInput(keyMap[e.key], 0); // press
            }
        });

        document.addEventListener('keyup', (e) => {
            if (keyMap.hasOwnProperty(e.key)) {
                e.preventDefault();
                this.handleInput(keyMap[e.key], 1); // release
            }
        });
    }

    sendKey(keyName, type) {
        const keyMap = { up: 0, down: 1, left: 2, right: 3, ok: 4, back: 5 };
        const typeMap = { press: 0, release: 1 };
        this.handleInput(keyMap[keyName], typeMap[type]);
    }

    handleInput(key, type) {
        if (this.wasmInstance && this.wasmInstance.exports.on_input) {
            this.wasmInstance.exports.on_input(key, type);
        }
    }

    // ========================================================================
    // Render Loop
    // ========================================================================

    tick(time) {
        if (!this.running) return;

        // Limit to ~60fps
        if (time - this.lastTime >= 16) {
            this.lastTime = time;

            // Clear the framebuffer
            this.canvasClear();

            // Call the WASM app's render function
            if (this.wasmInstance && this.wasmInstance.exports.render) {
                this.wasmInstance.exports.render();
            }

            // Present framebuffer to canvas
            this.present();
        }

        requestAnimationFrame((t) => this.tick(t));
    }

    present() {
        // Clear canvas
        this.ctx.fillStyle = '#000';
        this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);

        // Draw framebuffer with scaling
        this.ctx.fillStyle = '#0f0'; // Green for that classic look
        for (let y = 0; y < this.height; y++) {
            for (let x = 0; x < this.width; x++) {
                if (this.framebuffer[y * this.width + x]) {
                    this.ctx.fillRect(
                        x * this.scale,
                        y * this.scale,
                        this.scale,
                        this.scale
                    );
                }
            }
        }
    }

    // ========================================================================
    // Screenshot for Visual Testing
    // ========================================================================

    getFramebufferPNG() {
        // Create a temporary canvas at 1:1 scale for screenshot
        const tempCanvas = document.createElement('canvas');
        tempCanvas.width = this.width;
        tempCanvas.height = this.height;
        const tempCtx = tempCanvas.getContext('2d');

        // Draw framebuffer at 1:1 scale (black background, white pixels)
        const imageData = tempCtx.createImageData(this.width, this.height);
        for (let i = 0; i < this.framebuffer.length; i++) {
            const idx = i * 4;
            const value = this.framebuffer[i] ? 255 : 0;
            imageData.data[idx] = value;     // R
            imageData.data[idx + 1] = value; // G
            imageData.data[idx + 2] = value; // B
            imageData.data[idx + 3] = 255;   // A
        }
        tempCtx.putImageData(imageData, 0, 0);

        return tempCanvas.toDataURL('image/png');
    }
}
