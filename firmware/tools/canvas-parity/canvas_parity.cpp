// Canvas parity tester: runs each primitive against firmware/src/canvas.cpp
// and diffs the result against the Rust-generated golden framebuffer in
// tests/canvas-golden/<case>.bin. Both implementations must produce the
// same 8192 bytes for every case.
//
// Each case here mirrors one in tools/canvas-parity-gen/src/main.rs. Keep
// the two files in lockstep when adding new cases.
//
// Build:
//   cc -I ../../src -std=c++17 -x c++ \
//      ../../src/canvas.cpp canvas_parity.cpp -o canvas_parity
// Run from repo root:
//   firmware/tools/canvas-parity/canvas_parity

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <string>
#include <vector>
#include <functional>

#include "../../src/canvas.h"

using fri3d::Canvas;
using fri3d::Color;

struct Case {
    const char* name;
    std::function<void(Canvas&)> draw;
};

// ---- Load golden -----------------------------------------------------------

static std::vector<uint8_t> load_golden(const std::string& name) {
    const std::string path = "tests/canvas-golden/" + name + ".bin";
    std::ifstream f(path, std::ios::binary);
    if (!f) {
        std::fprintf(stderr, "[parity] cannot open %s\n", path.c_str());
        std::exit(1);
    }
    std::vector<uint8_t> data((std::istreambuf_iterator<char>(f)), {});
    if (data.size() != fri3d::SCREEN_WIDTH * fri3d::SCREEN_HEIGHT) {
        std::fprintf(stderr, "[parity] %s: wrong size %zu\n", path.c_str(), data.size());
        std::exit(1);
    }
    return data;
}

// ---- Diff helpers ----------------------------------------------------------

static void print_diff(const uint8_t* got, const uint8_t* want) {
    int first_diff = -1, count = 0;
    for (uint32_t i = 0; i < fri3d::SCREEN_WIDTH * fri3d::SCREEN_HEIGHT; ++i) {
        if (got[i] != want[i]) {
            if (first_diff < 0) first_diff = (int)i;
            count++;
        }
    }
    std::fprintf(stderr, "  %d pixels differ; first at (%d, %d)\n",
        count,
        first_diff % (int)fri3d::SCREEN_WIDTH,
        first_diff / (int)fri3d::SCREEN_WIDTH);
}

// ---- Cases (mirror tools/canvas-parity-gen) --------------------------------

static std::vector<Case> make_cases() {
    std::vector<Case> cs;
    cs.push_back({"clear_empty", [](Canvas&) {}});

    cs.push_back({"dot_single", [](Canvas& c) {
        c.set_color(Color::Black);
        c.draw_dot(3, 4);
    }});
    cs.push_back({"dot_oob", [](Canvas& c) {
        c.set_color(Color::Black);
        c.draw_dot(-1, -1);
        c.draw_dot(128, 64);
        c.draw_dot(200, 200);
    }});

    cs.push_back({"line_horizontal",       [](Canvas& c) { c.set_color(Color::Black); c.draw_line(10, 20, 60, 20); }});
    cs.push_back({"line_vertical",         [](Canvas& c) { c.set_color(Color::Black); c.draw_line(40, 5, 40, 50); }});
    cs.push_back({"line_diagonal_down_right",[](Canvas& c) { c.set_color(Color::Black); c.draw_line(0, 0, 127, 63); }});
    cs.push_back({"line_diagonal_up_left", [](Canvas& c) { c.set_color(Color::Black); c.draw_line(127, 63, 0, 0); }});

    cs.push_back({"frame_full",            [](Canvas& c) { c.set_color(Color::Black); c.draw_frame(0, 0, 128, 64); }});
    cs.push_back({"frame_small_centered",  [](Canvas& c) { c.set_color(Color::Black); c.draw_frame(40, 16, 48, 32); }});

    cs.push_back({"box_full",              [](Canvas& c) { c.set_color(Color::Black); c.draw_box(0, 0, 128, 64); }});
    cs.push_back({"box_small",             [](Canvas& c) { c.set_color(Color::Black); c.draw_box(20, 10, 30, 20); }});

    cs.push_back({"circle_center",         [](Canvas& c) { c.set_color(Color::Black); c.draw_circle(64, 32, 15); }});
    cs.push_back({"circle_r0",             [](Canvas& c) { c.set_color(Color::Black); c.draw_circle(64, 32, 0); }});

    cs.push_back({"disc_center",           [](Canvas& c) { c.set_color(Color::Black); c.draw_disc(64, 32, 15); }});
    cs.push_back({"disc_r0",               [](Canvas& c) { c.set_color(Color::Black); c.draw_disc(64, 32, 0); }});

    cs.push_back({"rframe_5r",             [](Canvas& c) { c.set_color(Color::Black); c.draw_rframe(10, 10, 60, 40, 5); }});
    cs.push_back({"rframe_r0_equals_frame",[](Canvas& c) { c.set_color(Color::Black); c.draw_rframe(10, 10, 60, 40, 0); }});
    cs.push_back({"rframe_large_radius_clamps",[](Canvas& c) { c.set_color(Color::Black); c.draw_rframe(10, 10, 20, 20, 100); }});
    cs.push_back({"rbox_5r",               [](Canvas& c) { c.set_color(Color::Black); c.draw_rbox(10, 10, 60, 40, 5); }});
    cs.push_back({"rbox_r0_equals_box",    [](Canvas& c) { c.set_color(Color::Black); c.draw_rbox(10, 10, 60, 40, 0); }});
    cs.push_back({"xor_disc_over_box",     [](Canvas& c) {
        c.set_color(Color::Black); c.draw_box(20, 10, 60, 40);
        c.set_color(Color::Xor);   c.draw_disc(50, 30, 12);
    }});
    cs.push_back({"xor_circle_over_box",   [](Canvas& c) {
        c.set_color(Color::Black); c.draw_box(20, 10, 60, 40);
        c.set_color(Color::Xor);   c.draw_circle(50, 30, 12);
    }});

    return cs;
}

// ---- Runner ----------------------------------------------------------------

int main() {
    auto cases = make_cases();
    int passed = 0, failed = 0;

    for (const auto& cs : cases) {
        Canvas c;
        c.clear();
        cs.draw(c);

        auto want = load_golden(cs.name);
        const uint8_t* got = c.buffer();
        if (std::memcmp(got, want.data(), want.size()) == 0) {
            std::printf("  [pass] %s\n", cs.name);
            passed++;
        } else {
            std::printf("  [FAIL] %s\n", cs.name);
            print_diff(got, want.data());
            failed++;
        }
    }

    std::printf("\n%d/%d passed\n", passed, passed + failed);
    return failed ? 1 : 0;
}
