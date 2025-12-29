//! Input types for WASM apps

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

impl TryFrom<u32> for InputKey {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
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
    Press = 0,
    Release = 1,
    Short = 2,
    Long = 3,
    Repeat = 4,
}

impl TryFrom<u32> for InputType {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
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
