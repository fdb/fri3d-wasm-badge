#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum InputKey {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    Ok = 4,
    Back = 5,
}

impl InputKey {
    pub const COUNT: usize = 6;
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
    Xor = 2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Font {
    Primary = 0,
    Secondary = 1,
    Keyboard = 2,
    BigNumbers = 3,
}
