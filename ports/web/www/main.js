// Fri3d Badge Web Emulator

import init, { Runtime } from './pkg/fri3d_web.js';

let runtime;
let currentApp;
let animationId;

// Key mapping
const keyMap = {
    'ArrowUp': 0,
    'ArrowDown': 1,
    'ArrowLeft': 2,
    'ArrowRight': 3,
    'Enter': 4,
    'Backspace': 5,
};

// Initialize the runtime
async function initRuntime() {
    await init();
    runtime = new Runtime();
    console.log('Runtime initialized');
}

// Load a WASM app
async function loadApp(name) {
    try {
        const response = await fetch(`/apps/${name}.wasm`);
        if (!response.ok) {
            throw new Error(`Failed to load ${name}.wasm`);
        }
        const bytes = await response.arrayBuffer();

        // Create imports that call into the runtime
        const imports = {
            env: {
                canvas_clear: () => runtime.canvas_clear(),
                canvas_width: () => runtime.canvas_width(),
                canvas_height: () => runtime.canvas_height(),
                canvas_set_color: (c) => runtime.canvas_set_color(c),
                canvas_set_font: (f) => runtime.canvas_set_font(f),
                canvas_draw_dot: (x, y) => runtime.canvas_draw_dot(x, y),
                canvas_draw_line: (x0, y0, x1, y1) => runtime.canvas_draw_line(x0, y0, x1, y1),
                canvas_draw_frame: (x, y, w, h) => runtime.canvas_draw_frame(x, y, w, h),
                canvas_draw_box: (x, y, w, h) => runtime.canvas_draw_box(x, y, w, h),
                canvas_draw_rframe: (x, y, w, h, r) => runtime.canvas_draw_rframe(x, y, w, h, r),
                canvas_draw_rbox: (x, y, w, h, r) => runtime.canvas_draw_rbox(x, y, w, h, r),
                canvas_draw_circle: (x, y, r) => runtime.canvas_draw_circle(x, y, r),
                canvas_draw_disc: (x, y, r) => runtime.canvas_draw_disc(x, y, r),
                random_seed: (s) => runtime.random_seed(s),
                random_get: () => runtime.random_get(),
                random_range: (m) => runtime.random_range(m),
                // Launcher host function (stub)
                launch_app: (index) => {
                    console.log('Launch app:', index);
                },
            }
        };

        const module = await WebAssembly.instantiate(bytes, imports);
        return module.instance.exports;
    } catch (error) {
        console.error('Error loading app:', error);
        return null;
    }
}

// Draw the canvas buffer to the display
function drawBuffer(ctx, buffer) {
    const imageData = ctx.createImageData(128, 64);

    for (let y = 0; y < 64; y++) {
        for (let x = 0; x < 128; x++) {
            const byteIndex = Math.floor((y * 128 + x) / 8);
            const bitIndex = 7 - (x % 8);
            const pixel = (buffer[byteIndex] >> bitIndex) & 1;

            const i = (y * 128 + x) * 4;

            if (pixel === 1) {
                // Black pixel
                imageData.data[i] = 0x1A;
                imageData.data[i + 1] = 0x1A;
                imageData.data[i + 2] = 0x2E;
            } else {
                // White pixel (badge color)
                imageData.data[i] = 0xE7;
                imageData.data[i + 1] = 0xD3;
                imageData.data[i + 2] = 0x96;
            }
            imageData.data[i + 3] = 255;
        }
    }

    ctx.putImageData(imageData, 0, 0);
}

// Main loop
function gameLoop(ctx, startTime) {
    return function frame(timestamp) {
        const timeMs = timestamp - startTime;

        // Update input
        runtime.update_input(timeMs);

        // Handle input events
        let event;
        while ((event = runtime.poll_event()) !== 0xFFFFFFFF) {
            const key = (event >> 8) & 0xFF;
            const type = event & 0xFF;

            // Only pass Short (2), Long (3), Repeat (4) to app
            if (type >= 2 && currentApp && currentApp.on_input) {
                currentApp.on_input(key, type);
            }
        }

        // Clear and render
        runtime.canvas_clear();
        if (currentApp && currentApp.render) {
            currentApp.render();
        }

        // Draw to canvas
        const buffer = runtime.get_buffer();
        drawBuffer(ctx, buffer);

        animationId = requestAnimationFrame(frame);
    };
}

// Keyboard event handlers
function setupKeyboard() {
    document.addEventListener('keydown', (e) => {
        const key = keyMap[e.code];
        if (key !== undefined) {
            runtime.key_down(key, performance.now());
            e.preventDefault();
        }
    });

    document.addEventListener('keyup', (e) => {
        const key = keyMap[e.code];
        if (key !== undefined) {
            runtime.key_up(key, performance.now());
            e.preventDefault();
        }
    });
}

// Main entry point
async function main() {
    const canvas = document.getElementById('display');
    const ctx = canvas.getContext('2d');

    // Initialize runtime
    await initRuntime();

    // Setup keyboard
    setupKeyboard();

    // Load default app
    currentApp = await loadApp('circles');

    // Setup app selector
    document.getElementById('load-btn').addEventListener('click', async () => {
        const appName = document.getElementById('app-select').value;
        if (animationId) {
            cancelAnimationFrame(animationId);
        }
        currentApp = await loadApp(appName);
        if (currentApp) {
            animationId = requestAnimationFrame(gameLoop(ctx, performance.now()));
        }
    });

    // Start game loop
    if (currentApp) {
        const startTime = performance.now();
        animationId = requestAnimationFrame(gameLoop(ctx, startTime));
    }
}

main().catch(console.error);
