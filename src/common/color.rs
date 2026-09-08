use crate::def_enum;
use enum_map::Enum;
use std::ops::Not;

def_enum! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Enum)]
    pub enum Color : u8 {
        White,
        Black
    }
}

impl Color {
    #[inline]
    pub const fn signum(self) -> i8 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }
}

impl Not for Color {
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ColorParseError;

impl From<Color> for char {
    #[inline]
    fn from(color: Color) -> Self {
        match color {
            Color::White => 'w',
            Color::Black => 'b',
        }
    }
}

impl TryFrom<char> for Color {
    type Error = ColorParseError;

    #[inline]
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c.to_ascii_lowercase() {
            'w' => Ok(Color::White),
            'b' => Ok(Color::Black),
            _ => Err(ColorParseError),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::Color;

    #[test]
    fn color_flip() {
        assert_eq!(!Color::White, Color::Black);
        assert_eq!(!Color::Black, Color::White);
    }
}
