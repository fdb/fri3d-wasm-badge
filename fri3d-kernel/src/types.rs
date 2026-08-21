#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum InputKey {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    Ok = 4,
    Back = 5,
    /// Kernel-reserved home key. Apps see it, but a short press always
    /// returns to the launcher before the app gets a say.
    Menu = 6,
}

impl InputKey {
    pub const COUNT: usize = 7;

    pub const fn from_index(index: usize) -> Self {
        match index {
            0 => InputKey::Up,
            1 => InputKey::Down,
            2 => InputKey::Left,
            3 => InputKey::Right,
            4 => InputKey::Ok,
            5 => InputKey::Back,
            _ => InputKey::Menu,
        }
    }

    pub fn from_u32(value: u32) -> Option<Self> {
        if (value as usize) < Self::COUNT {
            Some(Self::from_index(value as usize))
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum InputType {
    Press = 0,
    Release = 1,
    ShortPress = 2,
    LongPress = 3,
    Repeat = 4,
}

/// A DB32 palette index, 0..=31. See [`crate::palette`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Color(pub u8);

impl Color {
    pub const COUNT: u8 = 32;

    /// Clamp any byte to a valid index; out-of-range values become ink.
    pub const fn from_index(index: u8) -> Self {
        if index < Self::COUNT {
            Color(index)
        } else {
            crate::palette::INK
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Font {
    Primary = 0,
    Secondary = 1,
    Keyboard = 2,
    BigNumbers = 3,
    Title = 4,
}
