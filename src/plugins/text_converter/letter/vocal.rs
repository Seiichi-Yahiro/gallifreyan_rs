use super::Decorated;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Vocal {
    A,
    E,
    I,
    O,
    U,
}

impl fmt::Display for Vocal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Vocal::A => "A",
            Vocal::E => "E",
            Vocal::I => "I",
            Vocal::O => "O",
            Vocal::U => "U",
        };
        write!(f, "{}", s)
    }
}

impl TryFrom<&str> for Vocal {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let vocal = match value.to_ascii_lowercase().as_str() {
            "a" => Self::A,
            "e" => Self::E,
            "i" => Self::I,
            "o" => Self::O,
            "u" => Self::U,
            _ => return Err(format!("'{}' it is not a valid vocal!", value)),
        };

        Ok(vocal)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VocalPlacement {
    Inside,
    OnLine,
    Outside,
}

impl From<Vocal> for VocalPlacement {
    fn from(value: Vocal) -> Self {
        match value {
            Vocal::A => Self::Outside,
            Vocal::O => Self::Inside,
            Vocal::E | Vocal::I | Vocal::U => Self::OnLine,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VocalDecoration {
    None,
    LineInside,
    LineOutside,
}

impl From<Vocal> for VocalDecoration {
    fn from(value: Vocal) -> Self {
        match value {
            Vocal::I => Self::LineInside,
            Vocal::U => Self::LineOutside,
            Vocal::A | Vocal::E | Vocal::O => Self::None,
        }
    }
}

impl Decorated for VocalDecoration {
    fn dots(&self) -> usize {
        0
    }

    fn lines(&self) -> usize {
        match self {
            Self::LineOutside | Self::LineInside => 1,
            _ => 0,
        }
    }
}
