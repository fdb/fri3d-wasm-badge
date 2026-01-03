use crate::trace;
use crate::types::{InputKey, InputType};
use std::collections::VecDeque;

pub const INPUT_LONG_PRESS_MS: u32 = 300;
pub const INPUT_REPEAT_START_MS: u32 = INPUT_LONG_PRESS_MS;
pub const INPUT_REPEAT_INTERVAL_MS: u32 = 150;
pub const INPUT_RESET_COMBO_MS: u32 = 500;
const INPUT_EVENT_QUEUE_SIZE: usize = 16;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InputEvent {
    pub key: InputKey,
    pub kind: InputType,
}

pub trait InputBackend {
    fn poll(&mut self);
    fn has_event(&self) -> bool;
    fn next_event(&mut self) -> Option<InputEvent>;
}

#[derive(Copy, Clone, Debug)]
struct KeyState {
    pressed: bool,
    press_time: u32,
    long_press_fired: bool,
    last_repeat_time: u32,
}

impl Default for KeyState {
    fn default() -> Self {
        Self {
            pressed: false,
            press_time: 0,
            long_press_fired: false,
            last_repeat_time: 0,
        }
    }
}

pub struct InputManager {
    key_states: [KeyState; InputKey::COUNT],
    queue: VecDeque<InputEvent>,
    combo_start_time: u32,
    combo_active: bool,
    reset_callback: Option<Box<dyn FnMut()>>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            key_states: [KeyState::default(); InputKey::COUNT],
            queue: VecDeque::with_capacity(INPUT_EVENT_QUEUE_SIZE),
            combo_start_time: 0,
            combo_active: false,
            reset_callback: None,
        }
    }

    pub fn set_reset_callback<F>(&mut self, callback: F)
    where
        F: FnMut() + 'static,
    {
        self.reset_callback = Some(Box::new(callback));
    }

    pub fn update<B: InputBackend>(&mut self, backend: &mut B, time_ms: u32) {
        backend.poll();

        while backend.has_event() {
            let Some(raw_event) = backend.next_event() else {
                break;
            };

            let key_index = raw_event.key as usize;
            if key_index >= InputKey::COUNT {
                continue;
            }

            match raw_event.kind {
                InputType::Press => {
                    {
                        let state = &mut self.key_states[key_index];
                        state.pressed = true;
                        state.press_time = time_ms;
                        state.long_press_fired = false;
                        state.last_repeat_time = time_ms;
                    }
                    self.queue_event(raw_event.key, InputType::Press);
                }
                InputType::Release => {
                    let (queue_long, queue_short) = {
                        let state = &mut self.key_states[key_index];
                        if state.pressed {
                            let hold_time = time_ms.saturating_sub(state.press_time);
                            let queue_long =
                                !state.long_press_fired && hold_time >= INPUT_LONG_PRESS_MS;
                            let queue_short =
                                !state.long_press_fired && hold_time < INPUT_LONG_PRESS_MS;
                            state.pressed = false;
                            (queue_long, queue_short)
                        } else {
                            (false, false)
                        }
                    };
                    if queue_long {
                        self.queue_event(raw_event.key, InputType::LongPress);
                    } else if queue_short {
                        self.queue_event(raw_event.key, InputType::ShortPress);
                    }
                    self.queue_event(raw_event.key, InputType::Release);
                }
                _ => {}
            }
        }

        let mut pending_events = Vec::new();
        for (idx, state) in self.key_states.iter_mut().enumerate() {
            if state.pressed && !state.long_press_fired {
                let hold_time = time_ms.saturating_sub(state.press_time);
                if hold_time >= INPUT_LONG_PRESS_MS {
                    state.long_press_fired = true;
                    state.last_repeat_time = time_ms;
                    pending_events.push((InputKey::from_index(idx), InputType::LongPress));
                }
            }

            if state.pressed && state.long_press_fired {
                let since_repeat = time_ms.saturating_sub(state.last_repeat_time);
                if since_repeat >= INPUT_REPEAT_INTERVAL_MS {
                    state.last_repeat_time = time_ms;
                    pending_events.push((InputKey::from_index(idx), InputType::Repeat));
                }
            }
        }
        for (key, kind) in pending_events {
            self.queue_event(key, kind);
        }

        self.check_reset_combo(time_ms);
    }

    pub fn has_event(&self) -> bool {
        !self.queue.is_empty()
    }

    pub fn next_event(&mut self) -> Option<InputEvent> {
        self.queue.pop_front()
    }

    fn queue_event(&mut self, key: InputKey, kind: InputType) {
        if self.queue.len() >= INPUT_EVENT_QUEUE_SIZE {
            return;
        }
        trace::trace_call(
            "input_event",
            &[trace::TraceArg::Int(key as i64), trace::TraceArg::Int(kind as i64)],
        );
        self.queue.push_back(InputEvent { key, kind });
    }

    fn check_reset_combo(&mut self, time_ms: u32) {
        let left_held = self.key_states[InputKey::Left as usize].pressed;
        let back_held = self.key_states[InputKey::Back as usize].pressed;

        if left_held && back_held {
            if !self.combo_active {
                self.combo_active = true;
                self.combo_start_time = time_ms;
            } else if time_ms.saturating_sub(self.combo_start_time) >= INPUT_RESET_COMBO_MS {
                if let Some(callback) = self.reset_callback.as_mut() {
                    callback();
                }
                self.combo_active = false;
                self.combo_start_time = 0;
            }
        } else {
            self.combo_active = false;
            self.combo_start_time = 0;
        }
    }
}

impl InputKey {
    fn from_index(index: usize) -> Self {
        match index {
            0 => InputKey::Up,
            1 => InputKey::Down,
            2 => InputKey::Left,
            3 => InputKey::Right,
            4 => InputKey::Ok,
            _ => InputKey::Back,
        }
    }
}
