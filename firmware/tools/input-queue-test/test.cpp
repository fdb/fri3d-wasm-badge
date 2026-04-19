// Unit tests for firmware/src/input_queue.{h,cpp}.
//
// The queue's job is to preserve input events across long render frames:
// a button-polling task on hardware (or a JS event handler on the web)
// pushes events the instant they're detected, and the main loop drains
// them later. The fast Mandelbrot issue is exactly this — a press + release
// during a 500ms render needs to survive until render ends.

#include <cstdio>
#include <cassert>
#include "../../src/input_queue.h"

using fri3d::InputQueue;
using fri3d::InputEvent;

static int tests_run = 0, tests_failed = 0;
#define CHECK(cond) do { \
    tests_run++; \
    if (!(cond)) { tests_failed++; std::fprintf(stderr, "FAIL %s:%d: %s\n", __FILE__, __LINE__, #cond); } \
} while (0)

static void test_empty_pop_returns_false() {
    InputQueue q;
    InputEvent e{};
    CHECK(!q.pop(e));
    CHECK(q.size() == 0);
}

static void test_push_pop_fifo() {
    InputQueue q;
    CHECK(q.push({4, 0}));  // OK, PRESS
    CHECK(q.push({4, 1}));  // OK, RELEASE
    CHECK(q.push({4, 2}));  // OK, SHORT_PRESS
    CHECK(q.size() == 3);
    InputEvent e{};
    CHECK(q.pop(e)); CHECK(e.key == 4 && e.type == 0);
    CHECK(q.pop(e)); CHECK(e.key == 4 && e.type == 1);
    CHECK(q.pop(e)); CHECK(e.key == 4 && e.type == 2);
    CHECK(!q.pop(e));
}

// The core bug: if a press+release pair happens between two "render" cycles,
// the queue must hold both events so the next drain delivers them.
static void test_press_release_survives_busy_period() {
    InputQueue q;
    // Imagine: render #1 ends here.
    InputEvent e{};
    CHECK(!q.pop(e));

    // During the long render #2, a button task pushes a full press/release
    // cycle. Order must be preserved.
    CHECK(q.push({5, 0})); // BACK PRESS
    CHECK(q.push({5, 1})); // BACK RELEASE
    CHECK(q.push({5, 2})); // BACK SHORT_PRESS

    // Render #2 ends, main loop drains.
    CHECK(q.pop(e) && e.key == 5 && e.type == 0);
    CHECK(q.pop(e) && e.key == 5 && e.type == 1);
    CHECK(q.pop(e) && e.key == 5 && e.type == 2);
    CHECK(!q.pop(e));
}

static void test_overflow_returns_false_without_corruption() {
    InputQueue q;
    // CAPACITY - 1 is the real usable slot count for SPSC ring buffers
    // with head==tail meaning "empty".
    for (size_t i = 0; i < InputQueue::CAPACITY - 1; ++i) {
        CHECK(q.push({(uint32_t)i, 0}));
    }
    CHECK(!q.push({999, 9}));   // full — should refuse, not corrupt
    InputEvent e{};
    CHECK(q.pop(e) && e.key == 0);
    CHECK(q.push({999, 9}));    // now fits
    // Drain remaining + verify the new one comes last.
    for (size_t i = 1; i < InputQueue::CAPACITY - 1; ++i) {
        CHECK(q.pop(e) && e.key == (uint32_t)i);
    }
    CHECK(q.pop(e) && e.key == 999);
    CHECK(!q.pop(e));
}

static void test_wrap_around() {
    InputQueue q;
    InputEvent e{};
    // Fill, drain, fill, drain — forces the ring to wrap its indices.
    for (int cycle = 0; cycle < 10; ++cycle) {
        for (size_t i = 0; i < 10; ++i) {
            CHECK(q.push({(uint32_t)(cycle * 100 + i), 0}));
        }
        for (size_t i = 0; i < 10; ++i) {
            CHECK(q.pop(e) && e.key == (uint32_t)(cycle * 100 + i));
        }
        CHECK(!q.pop(e));
    }
}

int main() {
    test_empty_pop_returns_false();
    test_push_pop_fifo();
    test_press_release_survives_busy_period();
    test_overflow_returns_false_without_corruption();
    test_wrap_around();
    std::printf("%d/%d tests passed\n", tests_run - tests_failed, tests_run);
    return tests_failed;
}
