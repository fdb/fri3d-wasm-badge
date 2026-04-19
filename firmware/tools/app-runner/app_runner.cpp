// Native CLI that loads a .wasm app, runs N render frames through the same
// wasm3 + canvas pipeline as the firmware, and dumps the resulting 128×64
// framebuffer as a PNG or raw .bin.
//
// Sharing the host with the real firmware means any bug we find here also
// exists on the badge, and any fix we make here propagates to the badge
// automatically via CLAUDE.md's browser-first / offline-first workflow.
//
// Usage:
//   app_runner <wasm_path>
//              [--frames N]     number of render() calls (default 1)
//              [--seed S]       MT19937 seed (default 42)
//              [--out out.png]  framebuffer output (default fb.png)
//              [--out-bin bin]  also dump raw 128x64 bytes (0/1) to file
//              [--bg hex]       hex RGB for "0" pixels (default 000000)
//              [--fg hex]       hex RGB for "1" pixels (default 00ff00)
//
// Build: see firmware/tools/app-runner/run.sh.

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cstdint>
#include <fstream>
#include <string>
#include <vector>

#define STB_IMAGE_WRITE_IMPLEMENTATION
#include "stb_image_write.h"

extern "C" {
#include "wasm3.h"
#include "m3_env.h"
}

#include "canvas.h"
#include "random.h"
#include "wasm_host.h"

using fri3d::Canvas;
using fri3d::Random;

static std::vector<uint8_t> read_file(const char* path) {
    std::ifstream f(path, std::ios::binary);
    if (!f) {
        std::fprintf(stderr, "cannot open %s\n", path);
        std::exit(1);
    }
    return std::vector<uint8_t>((std::istreambuf_iterator<char>(f)), {});
}

// "rrggbb" -> (r, g, b).
static void parse_hex_color(const char* s, uint8_t rgb[3]) {
    auto hex = [](char c) -> int {
        if (c >= '0' && c <= '9') return c - '0';
        if (c >= 'a' && c <= 'f') return 10 + c - 'a';
        if (c >= 'A' && c <= 'F') return 10 + c - 'A';
        return 0;
    };
    rgb[0] = (uint8_t)((hex(s[0]) << 4) | hex(s[1]));
    rgb[1] = (uint8_t)((hex(s[2]) << 4) | hex(s[3]));
    rgb[2] = (uint8_t)((hex(s[4]) << 4) | hex(s[5]));
}

struct Args {
    const char* wasm_path = nullptr;
    uint32_t frames = 1;
    uint32_t seed = 42;
    const char* out_png = "fb.png";
    const char* out_bin = nullptr;
    uint8_t bg[3] = {0, 0, 0};
    uint8_t fg[3] = {0, 0xFF, 0};
};

static Args parse_args(int argc, char** argv) {
    Args a;
    int i = 1;
    while (i < argc) {
        const char* arg = argv[i];
        auto need = [&](const char* name) -> const char* {
            if (i + 1 >= argc) {
                std::fprintf(stderr, "%s requires a value\n", name);
                std::exit(2);
            }
            return argv[++i];
        };
        if      (std::strcmp(arg, "--frames")  == 0) a.frames  = (uint32_t)std::atoi(need("--frames"));
        else if (std::strcmp(arg, "--seed")    == 0) a.seed    = (uint32_t)std::atoi(need("--seed"));
        else if (std::strcmp(arg, "--out")     == 0) a.out_png = need("--out");
        else if (std::strcmp(arg, "--out-bin") == 0) a.out_bin = need("--out-bin");
        else if (std::strcmp(arg, "--bg")      == 0) parse_hex_color(need("--bg"), a.bg);
        else if (std::strcmp(arg, "--fg")      == 0) parse_hex_color(need("--fg"), a.fg);
        else if (arg[0] == '-') {
            std::fprintf(stderr, "unknown flag %s\n", arg);
            std::exit(2);
        }
        else if (!a.wasm_path) a.wasm_path = arg;
        else {
            std::fprintf(stderr, "extra positional arg %s\n", arg);
            std::exit(2);
        }
        i++;
    }
    if (!a.wasm_path) {
        std::fprintf(stderr, "usage: app_runner <wasm> [flags]\n");
        std::exit(2);
    }
    return a;
}

// Expand the mono framebuffer to RGB using bg/fg. Each 0 byte -> bg, each 1 -> fg.
static std::vector<uint8_t> expand(const uint8_t* fb, const Args& a) {
    const uint32_t n = fri3d::SCREEN_WIDTH * fri3d::SCREEN_HEIGHT;
    std::vector<uint8_t> rgb(n * 3);
    for (uint32_t i = 0; i < n; ++i) {
        const uint8_t* c = fb[i] ? a.fg : a.bg;
        rgb[i * 3 + 0] = c[0];
        rgb[i * 3 + 1] = c[1];
        rgb[i * 3 + 2] = c[2];
    }
    return rgb;
}

int main(int argc, char** argv) {
    Args a = parse_args(argc, argv);

    auto bytes = read_file(a.wasm_path);
    std::printf("[app-runner] loaded %s (%zu bytes)\n", a.wasm_path, bytes.size());

    Canvas canvas;
    Random rng(a.seed);

    const char* err = wasm_host::init(canvas, rng, bytes.data(), (uint32_t)bytes.size());
    if (err) { std::fprintf(stderr, "wasm init: %s\n", err); return 1; }

    for (uint32_t f = 0; f < a.frames; ++f) {
        wasm_host::render();
    }

    const uint8_t* fb = canvas.buffer();

    if (a.out_bin) {
        std::ofstream f(a.out_bin, std::ios::binary);
        f.write((const char*)fb, (std::streamsize)(fri3d::SCREEN_WIDTH * fri3d::SCREEN_HEIGHT));
        std::printf("[app-runner] wrote %s\n", a.out_bin);
    }

    auto rgb = expand(fb, a);
    if (!stbi_write_png(a.out_png,
                        (int)fri3d::SCREEN_WIDTH, (int)fri3d::SCREEN_HEIGHT,
                        3, rgb.data(), (int)(fri3d::SCREEN_WIDTH * 3))) {
        std::fprintf(stderr, "stbi_write_png failed\n");
        return 1;
    }
    std::printf("[app-runner] wrote %s (%ux%u, %u frames, seed=%u)\n",
                a.out_png, fri3d::SCREEN_WIDTH, fri3d::SCREEN_HEIGHT, a.frames, a.seed);
    return 0;
}
