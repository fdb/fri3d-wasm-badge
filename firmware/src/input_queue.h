#pragma once

#include <stdint.h>
#include <stddef.h>
#include <atomic>

// SPSC (single-producer, single-consumer) lock-free ring buffer of input
// events. On hardware the producer is the button-polling FreeRTOS task and
// the consumer is the main loop before render. On the browser the producer
// is JS DOM event handlers and the consumer is web_render.
//
// Sized for the worst case (every button transitions every 5ms for a
// half-second render) with headroom — 32 slots is plenty for realistic
// human input, but push() returns false if the queue overflows so callers
// can log without corrupting state.

namespace fri3d {

struct InputEvent {
    uint32_t key;   // matches wasm_host::key::* constants
    uint32_t type;  // matches wasm_host::event::* constants
};

class InputQueue {
public:
    static constexpr size_t CAPACITY = 32;

    // Returns false if the queue is full (caller should drop + log).
    bool push(const InputEvent& e);

    // Non-blocking pop; returns false if empty.
    bool pop(InputEvent& out);

    size_t size() const;

private:
    // head is written by producer, read by consumer (release/acquire).
    // tail is written by consumer, read by producer.
    std::atomic<size_t> m_head{0};
    std::atomic<size_t> m_tail{0};
    InputEvent m_buf[CAPACITY]{};
};

} // namespace fri3d
