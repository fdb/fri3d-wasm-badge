#include "input_queue.h"

namespace fri3d {

bool InputQueue::push(const InputEvent& e) {
    // Producer-side: compute next head, fail if it would meet tail (full).
    const size_t head = m_head.load(std::memory_order_relaxed);
    const size_t next = (head + 1) % CAPACITY;
    if (next == m_tail.load(std::memory_order_acquire)) return false;
    m_buf[head] = e;
    m_head.store(next, std::memory_order_release);
    return true;
}

bool InputQueue::pop(InputEvent& out) {
    // Consumer-side: if tail == head, empty.
    const size_t tail = m_tail.load(std::memory_order_relaxed);
    if (tail == m_head.load(std::memory_order_acquire)) return false;
    out = m_buf[tail];
    m_tail.store((tail + 1) % CAPACITY, std::memory_order_release);
    return true;
}

size_t InputQueue::size() const {
    const size_t head = m_head.load(std::memory_order_acquire);
    const size_t tail = m_tail.load(std::memory_order_acquire);
    return (head + CAPACITY - tail) % CAPACITY;
}

} // namespace fri3d
