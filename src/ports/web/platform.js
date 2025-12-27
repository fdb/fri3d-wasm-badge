// ============================================================================
// Fri3d Badge Web Platform - Thin Layer
// ============================================================================
// All rendering is done in Zig (canvas.zig), compiled into the WASM app.
// This platform just:
// 1. Loads the WASM app
// 2. Calls render() each frame
// 3. Reads the framebuffer from WASM memory
// 4. Displays it on an HTML Canvas

class Fri3dPlatform {
    constructor(canvas) {
        this.canvas = canvas;
        this.ctx = canvas.getContext('2d');
        this.scale = 4; // Display scale (128x64 -> 512x256)

        // Display dimensions (must match Zig canvas.zig constants)
        this.width = 128;
        this.height = 64;

        // WASM app instance
        this.wasmInstance = null;
        this.wasmMemory = null;
        this.framebufferPtr = null;

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

            // Minimal imports - Zig handles all rendering internally
            const imports = {
                env: {
                    // No canvas_* imports needed!
                    // All rendering is done in Zig canvas.zig
                }
            };

            // Instantiate the WASM module
            const result = await WebAssembly.instantiate(bytes, imports);
            this.wasmInstance = result.instance;
            this.wasmMemory = this.wasmInstance.exports.memory;

            // Get framebuffer pointer from WASM
            if (this.wasmInstance.exports.canvas_get_framebuffer) {
                this.framebufferPtr = this.wasmInstance.exports.canvas_get_framebuffer();
                console.log(`Framebuffer at address: ${this.framebufferPtr}`);
            } else {
                console.error('WASM module missing canvas_get_framebuffer export');
            }

            console.log('App loaded successfully');
            console.log('Exports:', Object.keys(this.wasmInstance.exports));
        } catch (error) {
            console.error('Failed to load app:', error);
        }
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

            // Call the WASM app's render function
            // The app draws to its internal framebuffer (in Zig canvas.zig)
            if (this.wasmInstance && this.wasmInstance.exports.render) {
                this.wasmInstance.exports.render();
            }

            // Read framebuffer from WASM memory and display
            this.present();
        }

        requestAnimationFrame((t) => this.tick(t));
    }

    present() {
        if (!this.wasmMemory || this.framebufferPtr === null) return;

        // Read framebuffer from WASM linear memory
        const fb = new Uint8Array(
            this.wasmMemory.buffer,
            this.framebufferPtr,
            this.width * this.height
        );

        // Clear canvas
        this.ctx.fillStyle = '#000';
        this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);

        // Draw framebuffer with scaling
        this.ctx.fillStyle = '#0f0'; // Green for that classic look
        for (let y = 0; y < this.height; y++) {
            for (let x = 0; x < this.width; x++) {
                if (fb[y * this.width + x]) {
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
        if (!this.wasmMemory || this.framebufferPtr === null) return null;

        // Read framebuffer from WASM
        const fb = new Uint8Array(
            this.wasmMemory.buffer,
            this.framebufferPtr,
            this.width * this.height
        );

        // Create a temporary canvas at 1:1 scale
        const tempCanvas = document.createElement('canvas');
        tempCanvas.width = this.width;
        tempCanvas.height = this.height;
        const tempCtx = tempCanvas.getContext('2d');

        // Draw framebuffer at 1:1 scale (black background, white pixels)
        const imageData = tempCtx.createImageData(this.width, this.height);
        for (let i = 0; i < fb.length; i++) {
            const idx = i * 4;
            const value = fb[i] ? 255 : 0;
            imageData.data[idx] = value;     // R
            imageData.data[idx + 1] = value; // G
            imageData.data[idx + 2] = value; // B
            imageData.data[idx + 3] = 255;   // A
        }
        tempCtx.putImageData(imageData, 0, 0);

        return tempCanvas.toDataURL('image/png');
    }
}
