use super::super::{AnglePlacement, PositionData};
use crate::math::angle::Degree;
use strum_macros::EnumIter;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum Consonant {
    B,
    J,
    T,
    TH,
    PH,
    WH,
    GH,
    CH,
    K,
    SH,
    Y,
    D,
    L,
    R,
    Z,
    C,
    Q,
    G,
    N,
    V,
    QU,
    H,
    P,
    W,
    X,
    F,
    M,
    S,
    NG,
}

impl Consonant {
    pub fn radius(&self, word_radius: f32, number_of_letters: usize) -> f32 {
        (word_radius * 0.75) / (1.0 + number_of_letters as f32 / 2.0)
    }

    pub fn position_data(
        &self,
        word_radius: f32,
        number_of_letters: usize,
        index: usize,
    ) -> PositionData {
        let distance = match ConsonantPlacement::from(*self) {
            ConsonantPlacement::DeepCut => {
                word_radius - self.radius(word_radius, number_of_letters) * 0.75
            }
            ConsonantPlacement::Inside => {
                if number_of_letters > 1 {
                    word_radius - self.radius(word_radius, number_of_letters) * 1.5
                } else {
                    0.0
                }
            }
            ConsonantPlacement::ShallowCut => word_radius,
            ConsonantPlacement::OnLine => word_radius,
        };

        let angle = index as f32 * (360.0 / number_of_letters as f32);

        PositionData {
            distance,
            angle: Degree::new(angle),
            angle_placement: AnglePlacement::Relative,
        }
    }
}

impl TryFrom<&str> for Consonant {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let consonant = match value.to_ascii_lowercase().as_str() {
            "b" => Self::B,
            "j" => Self::J,
            "t" => Self::T,
            "th" => Self::TH,
            "ph" => Self::PH,
            "wh" => Self::WH,
            "gh" => Self::GH,
            "ch" => Self::CH,
            "k" => Self::K,
            "sh" => Self::SH,
            "y" => Self::Y,
            "d" => Self::D,
            "l" => Self::L,
            "r" => Self::R,
            "z" => Self::Z,
            "c" => Self::C,
            "q" => Self::Q,
            "g" => Self::G,
            "n" => Self::N,
            "v" => Self::V,
            "qu" => Self::QU,
            "h" => Self::H,
            "p" => Self::P,
            "w" => Self::W,
            "x" => Self::X,
            "f" => Self::F,
            "m" => Self::M,
            "s" => Self::S,
            "ng" => Self::NG,
            _ => return Err(format!("'{}' is not a valid consonant!", value)),
        };

        Ok(consonant)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ConsonantPlacement {
    DeepCut,
    Inside,
    ShallowCut,
    OnLine,
}

impl From<Consonant> for ConsonantPlacement {
    fn from(value: Consonant) -> Self {
        match value {
            Consonant::B
            | Consonant::CH
            | Consonant::D
            | Consonant::G
            | Consonant::H
            | Consonant::F => Self::DeepCut,
            Consonant::J
            | Consonant::PH
            | Consonant::K
            | Consonant::L
            | Consonant::C
            | Consonant::N
            | Consonant::P
            | Consonant::M => Self::Inside,
            Consonant::T
            | Consonant::WH
            | Consonant::SH
            | Consonant::R
            | Consonant::V
            | Consonant::W
            | Consonant::S => Self::ShallowCut,
            Consonant::TH
            | Consonant::GH
            | Consonant::Y
            | Consonant::Z
            | Consonant::Q
            | Consonant::QU
            | Consonant::X
            | Consonant::NG => Self::OnLine,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ConsonantDecoration {
    None,
    SingleDot,
    DoubleDot,
    TripleDot,
    QuadrupleDot,
    SingleLine,
    DoubleLine,
    TripleLine,
}

impl From<Consonant> for ConsonantDecoration {
    fn from(value: Consonant) -> Self {
        match value {
            Consonant::B | Consonant::J | Consonant::T | Consonant::TH => Self::None,
            Consonant::PH | Consonant::WH | Consonant::GH => Self::SingleDot,
            Consonant::CH | Consonant::K | Consonant::SH | Consonant::Y => Self::DoubleDot,
            Consonant::D | Consonant::L | Consonant::R | Consonant::Z => Self::TripleDot,
            Consonant::C | Consonant::Q => Self::QuadrupleDot,
            Consonant::G | Consonant::N | Consonant::V | Consonant::QU => Self::SingleLine,
            Consonant::H | Consonant::P | Consonant::W | Consonant::X => Self::DoubleLine,
            Consonant::F | Consonant::M | Consonant::S | Consonant::NG => Self::TripleLine,
        }
    }
}

impl ConsonantDecoration {
    pub fn dots(&self) -> usize {
        match self {
            Self::SingleDot => 1,
            Self::DoubleDot => 2,
            Self::TripleDot => 3,
            Self::QuadrupleDot => 4,
            _ => 0,
        }
    }

    pub fn lines(&self) -> usize {
        match self {
            Self::SingleLine => 1,
            Self::DoubleLine => 2,
            Self::TripleLine => 3,
            _ => 0,
        }
    }
}
