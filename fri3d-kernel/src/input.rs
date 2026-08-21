//! Input manager: turns raw press/release transitions into the event stream
//! apps consume (press, release, short, long, repeat).
//!
//! Fixed-size: one `KeyState` per key and a bounded queue. Events past the
//! queue cap are dropped, never allocated.

use crate::limits::INPUT_QUEUE;
use crate::types::{InputKey, InputType};
use heapless::Deque;

pub const INPUT_LONG_PRESS_MS: u32 = 300;
pub const INPUT_REPEAT_START_MS: u32 = INPUT_LONG_PRESS_MS;
pub const INPUT_REPEAT_INTERVAL_MS: u32 = 150;
pub const INPUT_RESET_COMBO_MS: u32 = 500;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InputEvent {
    pub key: InputKey,
    pub kind: InputType,
}

#[derive(Copy, Clone, Debug, Default)]
struct KeyState {
    pressed: bool,
    press_time: u32,
    long_press_fired: bool,
    last_repeat_time: u32,
}

pub struct InputManager {
    key_states: [KeyState; InputKey::COUNT],
    queue: Deque<InputEvent, INPUT_QUEUE>,
    combo_start_time: u32,
    combo_active: bool,
    reset_fired: bool,
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InputManager {
    pub const fn new() -> Self {
        Self {
            key_states: [KeyState {
                pressed: false,
                press_time: 0,
                long_press_fired: false,
                last_repeat_time: 0,
            }; InputKey::COUNT],
            queue: Deque::new(),
            combo_start_time: 0,
            combo_active: false,
            reset_fired: false,
        }
    }

    /// Feed one physical transition. Idempotent for repeated presses or
    /// releases of the same key, so hosts may call it from a debounced
    /// poll without tracking edges themselves.
    pub fn push_raw(&mut self, key: InputKey, pressed: bool, time_ms: u32) {
        let state = &mut self.key_states[key as usize];
        if pressed {
            if state.pressed {
                return;
            }
            state.pressed = true;
            state.press_time = time_ms;
            state.long_press_fired = false;
            state.last_repeat_time = time_ms;
            self.queue_event(key, InputType::Press);
        } else {
            if !state.pressed {
                return;
            }
            let hold_time = time_ms.saturating_sub(state.press_time);
            let long_fired = state.long_press_fired;
            state.pressed = false;
            if !long_fired {
                if hold_time >= INPUT_LONG_PRESS_MS {
                    self.queue_event(key, InputType::LongPress);
                } else {
                    self.queue_event(key, InputType::ShortPress);
                }
            }
            self.queue_event(key, InputType::Release);
        }
        // Start the combo clock at the press itself, not at the next poll.
        self.check_reset_combo(time_ms);
    }

    /// Advance time: synthesize long-press and repeat events for held keys
    /// and track the reset combo.
    pub fn update(&mut self, time_ms: u32) {
        for idx in 0..InputKey::COUNT {
            let key = InputKey::from_index(idx);
            let state = &mut self.key_states[idx];
            if !state.pressed {
                continue;
            }
            if !state.long_press_fired {
                let hold_time = time_ms.saturating_sub(state.press_time);
                if hold_time >= INPUT_LONG_PRESS_MS {
                    state.long_press_fired = true;
                    state.last_repeat_time = time_ms;
                    self.queue_event(key, InputType::LongPress);
                    continue;
                }
            } else {
                let since_repeat = time_ms.saturating_sub(state.last_repeat_time);
                if since_repeat >= INPUT_REPEAT_INTERVAL_MS {
                    state.last_repeat_time = time_ms;
                    self.queue_event(key, InputType::Repeat);
                }
            }
        }
        self.check_reset_combo(time_ms);
    }

    pub fn next_event(&mut self) -> Option<InputEvent> {
        self.queue.pop_front()
    }

    pub fn is_pressed(&self, key: InputKey) -> bool {
        self.key_states[key as usize].pressed
    }

    /// True once per completed Left+Back hold. Clears on read.
    pub fn take_reset_combo(&mut self) -> bool {
        core::mem::replace(&mut self.reset_fired, false)
    }

    fn queue_event(&mut self, key: InputKey, kind: InputType) {
        // Full queue: drop. The host polls faster than a human can type;
        // overflow means the app is stalled and stale input is worthless.
        let _ = self.queue.push_back(InputEvent { key, kind });
    }

    fn check_reset_combo(&mut self, time_ms: u32) {
        let left_held = self.key_states[InputKey::Left as usize].pressed;
        let back_held = self.key_states[InputKey::Back as usize].pressed;

        if left_held && back_held {
            if !self.combo_active {
                self.combo_active = true;
                self.combo_start_time = time_ms;
            } else if time_ms.saturating_sub(self.combo_start_time) >= INPUT_RESET_COMBO_MS {
                self.reset_fired = true;
                self.combo_active = false;
                self.combo_start_time = 0;
            }
        } else {
            self.combo_active = false;
            self.combo_start_time = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drain(m: &mut InputManager) -> alloc::vec::Vec<InputEvent> {
        let mut out = alloc::vec::Vec::new();
        while let Some(ev) = m.next_event() {
            out.push(ev);
        }
        out
    }

    #[test]
    fn short_press_sequence() {
        let mut m = InputManager::new();
        m.push_raw(InputKey::Ok, true, 0);
        m.update(10);
        m.push_raw(InputKey::Ok, false, 50);
        let kinds: alloc::vec::Vec<_> = drain(&mut m).iter().map(|e| e.kind).collect();
        assert_eq!(kinds, [InputType::Press, InputType::ShortPress, InputType::Release]);
    }

    #[test]
    fn long_press_then_repeats_then_release() {
        let mut m = InputManager::new();
        m.push_raw(InputKey::Down, true, 0);
        m.update(299);
        assert_eq!(drain(&mut m).len(), 1);
        m.update(300);
        assert_eq!(drain(&mut m), [InputEvent { key: InputKey::Down, kind: InputType::LongPress }]);
        m.update(449);
        assert!(drain(&mut m).is_empty());
        m.update(450);
        assert_eq!(drain(&mut m)[0].kind, InputType::Repeat);
        m.push_raw(InputKey::Down, false, 500);
        assert_eq!(drain(&mut m), [InputEvent { key: InputKey::Down, kind: InputType::Release }]);
    }

    #[test]
    fn duplicate_press_is_ignored() {
        let mut m = InputManager::new();
        m.push_raw(InputKey::Ok, true, 0);
        m.push_raw(InputKey::Ok, true, 5);
        assert_eq!(drain(&mut m).len(), 1);
    }

    #[test]
    fn reset_combo_fires_once() {
        let mut m = InputManager::new();
        m.push_raw(InputKey::Left, true, 0);
        m.push_raw(InputKey::Back, true, 0);
        m.update(499);
        assert!(!m.take_reset_combo());
        m.update(500);
        assert!(m.take_reset_combo());
        assert!(!m.take_reset_combo());
    }

    #[test]
    fn queue_is_bounded() {
        let mut m = InputManager::new();
        for i in 0..100u32 {
            m.push_raw(InputKey::Ok, i % 2 == 0, i);
        }
        assert!(drain(&mut m).len() <= INPUT_QUEUE);
    }
}
