// ============================================================================
// Fri3d Badge Web Platform
// ============================================================================
// Pure JavaScript platform implementation - no Emscripten needed!
// Uses browser's native WebAssembly API to run WASM apps.

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
        this.color = 0; // 0=black, 1=white, 2=xor
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
        // Simplified rounded frame - just draw regular frame for now
        this.drawFrame(x, y, w, h);
    }

    drawRBox(x, y, w, h, r) {
        // Simplified rounded box - just draw regular box for now
        this.drawBox(x, y, w, h);
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

    drawStr(x, y, strPtr) {
        // Read string from WASM memory
        if (!this.wasmMemory) return;
        const mem = new Uint8Array(this.wasmMemory.buffer);
        let str = '';
        let i = strPtr;
        while (mem[i] !== 0) {
            str += String.fromCharCode(mem[i]);
            i++;
        }

        // Simple 5x7 font rendering (placeholder)
        const charWidth = 6;
        for (let c = 0; c < str.length; c++) {
            this.drawCharPlaceholder(x + c * charWidth, y, str[c]);
        }
    }

    drawCharPlaceholder(x, y, char) {
        // Very simple placeholder - just draw a box for each char
        this.drawBox(x, y - 6, 5, 7);
    }

    stringWidth(strPtr) {
        // Read string length from WASM memory
        if (!this.wasmMemory) return 0;
        const mem = new Uint8Array(this.wasmMemory.buffer);
        let len = 0;
        let i = strPtr;
        while (mem[i] !== 0) {
            len++;
            i++;
        }
        return len * 6; // Assuming 6 pixels per character
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
}
