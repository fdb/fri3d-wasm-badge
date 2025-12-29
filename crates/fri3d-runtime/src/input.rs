//! Input handling with short/long press detection

/// Input keys
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum InputKey {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    Ok = 4,
    Back = 5,
}

impl TryFrom<u8> for InputKey {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(InputKey::Up),
            1 => Ok(InputKey::Down),
            2 => Ok(InputKey::Left),
            3 => Ok(InputKey::Right),
            4 => Ok(InputKey::Ok),
            5 => Ok(InputKey::Back),
            _ => Err(()),
        }
    }
}

/// Input event types
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum InputType {
    /// Key went down (raw)
    Press = 0,
    /// Key went up (raw)
    Release = 1,
    /// Released quickly (< 300ms)
    Short = 2,
    /// Held for 500ms+
    Long = 3,
    /// Held down, repeating
    Repeat = 4,
}

impl TryFrom<u8> for InputType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(InputType::Press),
            1 => Ok(InputType::Release),
            2 => Ok(InputType::Short),
            3 => Ok(InputType::Long),
            4 => Ok(InputType::Repeat),
            _ => Err(()),
        }
    }
}

/// Input event
#[derive(Clone, Copy, Debug)]
pub struct InputEvent {
    pub key: InputKey,
    pub event_type: InputType,
}

/// Timing constants
const SHORT_PRESS_MAX_MS: u32 = 300;
const LONG_PRESS_MS: u32 = 500;
const RESET_COMBO_MS: u32 = 500;
const REPEAT_DELAY_MS: u32 = 400;
const REPEAT_RATE_MS: u32 = 100;

const NUM_KEYS: usize = 6;
const EVENT_QUEUE_SIZE: usize = 16;

/// Per-key state
#[derive(Clone, Copy, Default)]
struct KeyState {
    /// Is the key currently pressed?
    pressed: bool,
    /// When was it pressed?
    press_time: u32,
    /// Has long press been fired?
    long_fired: bool,
    /// Last repeat time
    last_repeat: u32,
}

/// Input manager with short/long press detection
pub struct InputManager {
    key_states: [KeyState; NUM_KEYS],
    event_queue: [Option<InputEvent>; EVENT_QUEUE_SIZE],
    queue_head: usize,
    queue_tail: usize,
    reset_start_time: Option<u32>,
    reset_callback: Option<fn()>,
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InputManager {
    /// Create a new input manager
    pub fn new() -> Self {
        Self {
            key_states: [KeyState::default(); NUM_KEYS],
            event_queue: [None; EVENT_QUEUE_SIZE],
            queue_head: 0,
            queue_tail: 0,
            reset_start_time: None,
            reset_callback: None,
        }
    }

    /// Set callback for reset combo (Left + Back held for 500ms)
    pub fn set_reset_callback(&mut self, callback: fn()) {
        self.reset_callback = Some(callback);
    }

    /// Report a raw key press
    pub fn key_down(&mut self, key: InputKey, time_ms: u32) {
        let idx = key as usize;
        if !self.key_states[idx].pressed {
            self.key_states[idx].pressed = true;
            self.key_states[idx].press_time = time_ms;
            self.key_states[idx].long_fired = false;
            self.key_states[idx].last_repeat = time_ms;

            self.push_event(InputEvent {
                key,
                event_type: InputType::Press,
            });
        }
    }

    /// Report a raw key release
    pub fn key_up(&mut self, key: InputKey, time_ms: u32) {
        let idx = key as usize;
        if self.key_states[idx].pressed {
            // Copy values we need before modifying
            let press_time = self.key_states[idx].press_time;
            let long_fired = self.key_states[idx].long_fired;

            self.key_states[idx].pressed = false;

            self.push_event(InputEvent {
                key,
                event_type: InputType::Release,
            });

            // Generate short press if released quickly and long wasn't fired
            let duration = time_ms.saturating_sub(press_time);
            if duration < SHORT_PRESS_MAX_MS && !long_fired {
                self.push_event(InputEvent {
                    key,
                    event_type: InputType::Short,
                });
            }
        }
    }

    /// Update input state (call each frame)
    pub fn update(&mut self, time_ms: u32) {
        // Check for long press on each key
        for i in 0..NUM_KEYS {
            // Copy values we need
            let pressed = self.key_states[i].pressed;
            let long_fired = self.key_states[i].long_fired;
            let press_time = self.key_states[i].press_time;
            let last_repeat = self.key_states[i].last_repeat;

            if pressed && !long_fired {
                let duration = time_ms.saturating_sub(press_time);
                if duration >= LONG_PRESS_MS {
                    self.key_states[i].long_fired = true;
                    if let Ok(key) = InputKey::try_from(i as u8) {
                        self.push_event(InputEvent {
                            key,
                            event_type: InputType::Long,
                        });
                    }
                }
            }

            // Check for repeat (re-read long_fired in case it was just set)
            let long_fired = self.key_states[i].long_fired;
            if pressed && long_fired {
                let since_last = time_ms.saturating_sub(last_repeat);
                let delay = if last_repeat == press_time {
                    REPEAT_DELAY_MS
                } else {
                    REPEAT_RATE_MS
                };
                if since_last >= delay {
                    self.key_states[i].last_repeat = time_ms;
                    if let Ok(key) = InputKey::try_from(i as u8) {
                        self.push_event(InputEvent {
                            key,
                            event_type: InputType::Repeat,
                        });
                    }
                }
            }
        }

        // Check for reset combo (Left + Back)
        let left_pressed = self.key_states[InputKey::Left as usize].pressed;
        let back_pressed = self.key_states[InputKey::Back as usize].pressed;

        if left_pressed && back_pressed {
            if let Some(start) = self.reset_start_time {
                if time_ms.saturating_sub(start) >= RESET_COMBO_MS {
                    if let Some(callback) = self.reset_callback {
                        callback();
                    }
                    self.reset_start_time = None;
                }
            } else {
                self.reset_start_time = Some(time_ms);
            }
        } else {
            self.reset_start_time = None;
        }
    }

    /// Poll next event from queue
    pub fn poll_event(&mut self) -> Option<InputEvent> {
        if self.queue_head == self.queue_tail {
            return None;
        }

        let event = self.event_queue[self.queue_head].take();
        self.queue_head = (self.queue_head + 1) % EVENT_QUEUE_SIZE;
        event
    }

    /// Check if a key is currently pressed
    pub fn is_pressed(&self, key: InputKey) -> bool {
        self.key_states[key as usize].pressed
    }

    /// Push an event to the queue
    fn push_event(&mut self, event: InputEvent) {
        let next_tail = (self.queue_tail + 1) % EVENT_QUEUE_SIZE;
        if next_tail != self.queue_head {
            self.event_queue[self.queue_tail] = Some(event);
            self.queue_tail = next_tail;
        }
        // If queue is full, drop the event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_press() {
        let mut input = InputManager::new();

        input.key_down(InputKey::Ok, 0);
        input.key_up(InputKey::Ok, 100);

        // Should have Press, Release, Short
        assert!(matches!(
            input.poll_event(),
            Some(InputEvent {
                key: InputKey::Ok,
                event_type: InputType::Press
            })
        ));
        assert!(matches!(
            input.poll_event(),
            Some(InputEvent {
                key: InputKey::Ok,
                event_type: InputType::Release
            })
        ));
        assert!(matches!(
            input.poll_event(),
            Some(InputEvent {
                key: InputKey::Ok,
                event_type: InputType::Short
            })
        ));
        assert!(input.poll_event().is_none());
    }

    #[test]
    fn test_long_press() {
        let mut input = InputManager::new();

        input.key_down(InputKey::Ok, 0);
        input.update(600);
        input.key_up(InputKey::Ok, 600);

        // Should have Press, Long, Release (no Short because long was fired)
        assert!(matches!(
            input.poll_event(),
            Some(InputEvent {
                key: InputKey::Ok,
                event_type: InputType::Press
            })
        ));
        assert!(matches!(
            input.poll_event(),
            Some(InputEvent {
                key: InputKey::Ok,
                event_type: InputType::Long
            })
        ));
        assert!(matches!(
            input.poll_event(),
            Some(InputEvent {
                key: InputKey::Ok,
                event_type: InputType::Release
            })
        ));
        assert!(input.poll_event().is_none());
    }
}
