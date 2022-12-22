use crate::image_types::{
    new_stroke_mode, AnglePlacement, CircleChildren, LineSlotChildren, PositionData, Radius, Text,
};
use crate::math::Angle;
use crate::style::Styles;
use crate::svg_view::Interaction;
use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::DrawMode;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Reflect)]
#[reflect(Component)]
pub enum Letter {
    Vocal(Vocal),
    Consonant(Consonant),
}

impl Default for Letter {
    fn default() -> Self {
        Self::Consonant(Consonant::B)
    }
}

impl TryFrom<&str> for Letter {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Vocal::try_from(value)
            .map(Self::Vocal)
            .or_else(|_| Consonant::try_from(value).map(Self::Consonant))
            .map_err(|_| {
                format!(
                    "Cannot assign letter to '{}' as it is not a valid letter!",
                    value
                )
            })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Reflect, FromReflect)]
pub enum Vocal {
    A,
    E,
    I,
    O,
    U,
}

impl Vocal {
    pub fn radius(&self, word_radius: f32, number_of_letters: usize) -> f32 {
        (word_radius * 0.75 * 0.4) / (1.0 + number_of_letters as f32 / 2.0)
    }

    pub fn position_data(
        &self,
        word_radius: f32,
        number_of_letters: usize,
        index: usize,
    ) -> PositionData {
        let distance = match Placement::from(*self) {
            Placement::OnLine => word_radius,
            Placement::Outside => word_radius + self.radius(word_radius, number_of_letters) * 1.5,
            Placement::Inside => {
                if number_of_letters > 1 {
                    word_radius - self.radius(word_radius, number_of_letters) * 1.5
                } else {
                    0.0
                }
            }
            _ => {
                unreachable!("Vocals can only have a placement of OnLine, Outside or Inside!");
            }
        };

        let angle = index as f32 * (360.0 / number_of_letters as f32);

        PositionData {
            distance,
            angle: Angle::new_degree(angle),
            angle_placement: AnglePlacement::Relative,
        }
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
            _ => {
                return Err(format!(
                    "Cannot assign vocal to '{}' as it is not a valid vocal!",
                    value
                ))
            }
        };

        Ok(vocal)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Reflect, FromReflect)]
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
        let distance = match Placement::from(*self) {
            Placement::DeepCut => word_radius - self.radius(word_radius, number_of_letters) * 0.75,
            Placement::Inside => {
                if number_of_letters > 1 {
                    word_radius - self.radius(word_radius, number_of_letters) * 1.5
                } else {
                    0.0
                }
            }
            Placement::ShallowCut => word_radius,
            Placement::OnLine => word_radius,
            _ => {
                unreachable!(
                    "Consonants can only have placement of DeepCut, Inside, ShallowCut or OnLine!"
                );
            }
        };

        let angle = index as f32 * (360.0 / number_of_letters as f32);

        PositionData {
            distance,
            angle: Angle::new_degree(angle),
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
            _ => {
                return Err(format!(
                    "Cannot assign consonant to '{}' as it is not a valid consonant!",
                    value
                ))
            }
        };

        Ok(consonant)
    }
}

impl Letter {
    pub fn radius(&self, word_radius: f32, number_of_letters: usize) -> f32 {
        match self {
            Letter::Vocal(vocal) => vocal.radius(word_radius, number_of_letters),
            Letter::Consonant(consonant) => consonant.radius(word_radius, number_of_letters),
        }
    }

    pub fn position_data(
        &self,
        word_radius: f32,
        number_of_letters: usize,
        index: usize,
    ) -> PositionData {
        match self {
            Letter::Vocal(vocal) => vocal.position_data(word_radius, number_of_letters, index),
            Letter::Consonant(consonant) => {
                consonant.position_data(word_radius, number_of_letters, index)
            }
        }
    }
}

#[derive(Bundle)]
pub struct LetterBundle {
    pub letter: Letter,
    pub text: Text,
    pub radius: Radius,
    pub position_data: PositionData,
    pub placement: Placement,
    pub decoration: Decoration,
    pub dots: CircleChildren,
    pub line_slots: LineSlotChildren,
    pub interaction: Interaction,
}

impl LetterBundle {
    pub fn new(
        letter: Letter,
        letter_text: String,
        word_radius: f32,
        number_of_letters: usize,
        index: usize,
    ) -> Self {
        Self {
            letter,
            radius: Radius(letter.radius(word_radius, number_of_letters)),
            position_data: letter.position_data(word_radius, number_of_letters, index),
            placement: Placement::from(letter),
            decoration: Decoration::from(letter),
            dots: Default::default(),
            text: Text(letter_text),
            line_slots: Default::default(),
            interaction: Interaction::default(),
        }
    }
}

// needed for reflection
pub fn add_shape_for_letter(
    mut commands: Commands,
    query: Query<Entity, Added<Letter>>,
    styles: Res<Styles>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(ShapeBundle {
            mode: DrawMode::Stroke(new_stroke_mode(styles.svg_color)),
            transform: Transform::from_xyz(0.0, 0.0, 0.1),
            ..default()
        });
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Component, Reflect)]
#[reflect(Component)]
pub enum Placement {
    DeepCut,    // c
    Inside,     // cv
    ShallowCut, // c
    #[default]
    OnLine, // cv
    Outside,    // v
}

impl From<Letter> for Placement {
    fn from(value: Letter) -> Self {
        match value {
            Letter::Vocal(vocal) => Self::from(vocal),
            Letter::Consonant(consonant) => Self::from(consonant),
        }
    }
}

impl From<Vocal> for Placement {
    fn from(value: Vocal) -> Self {
        match value {
            Vocal::A => Self::Outside,
            Vocal::O => Self::Inside,
            Vocal::E | Vocal::I | Vocal::U => Self::OnLine,
        }
    }
}

impl From<Consonant> for Placement {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Component, Reflect)]
#[reflect(Component)]
pub enum Decoration {
    #[default]
    None, // cv
    SingleDot,    // c
    DoubleDot,    // c
    TripleDot,    // c
    QuadrupleDot, // c
    SingleLine,   // c
    DoubleLine,   // c
    TripleLine,   // c
    LineInside,   // v
    LineOutside,  // v
}

impl From<Letter> for Decoration {
    fn from(value: Letter) -> Self {
        match value {
            Letter::Vocal(vocal) => Self::from(vocal),
            Letter::Consonant(consonant) => Self::from(consonant),
        }
    }
}

impl From<Vocal> for Decoration {
    fn from(value: Vocal) -> Self {
        match value {
            Vocal::I => Self::LineInside,
            Vocal::U => Self::LineOutside,
            Vocal::A | Vocal::E | Vocal::O => Self::None,
        }
    }
}

impl From<Consonant> for Decoration {
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

impl Decoration {
    pub fn dots(&self) -> usize {
        match self {
            Decoration::SingleDot => 1,
            Decoration::DoubleDot => 2,
            Decoration::TripleDot => 3,
            Decoration::QuadrupleDot => 4,
            _ => 0,
        }
    }

    pub fn lines(&self) -> usize {
        match self {
            Decoration::SingleLine | Decoration::LineInside | Decoration::LineOutside => 1,
            Decoration::DoubleLine => 2,
            Decoration::TripleLine => 3,
            _ => 0,
        }
    }

    pub fn line_points_outside(&self) -> bool {
        *self == Decoration::LineOutside
    }
}
