use crate::image_types::{
    new_stroke_mode, AnglePlacement, CircleChildren, LineSlotChildren, PositionData, Radius,
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
    ConsonantWithVocal { consonant: Consonant, vocal: Vocal },
}

impl Default for Letter {
    fn default() -> Self {
        Self::Consonant(Consonant::B)
    }
}

impl Letter {
    pub fn is_cutting(&self) -> bool {
        match self {
            Self::Consonant(consonant) | Self::ConsonantWithVocal { consonant, .. } => {
                match ConsonantPlacement::from(*consonant) {
                    ConsonantPlacement::DeepCut | ConsonantPlacement::ShallowCut => true,
                    ConsonantPlacement::OnLine | ConsonantPlacement::Inside => false,
                }
            }
            Self::Vocal(_) => false,
        }
    }

    pub fn dots(&self) -> usize {
        match self {
            Letter::Vocal(vocal) => VocalDecoration::from(*vocal).dots(),
            Letter::Consonant(consonant) | Letter::ConsonantWithVocal { consonant, .. } => {
                ConsonantDecoration::from(*consonant).dots()
            }
        }
    }

    pub fn lines(&self) -> usize {
        match self {
            Letter::Vocal(vocal) => VocalDecoration::from(*vocal).lines(),
            Letter::Consonant(consonant) | Letter::ConsonantWithVocal { consonant, .. } => {
                ConsonantDecoration::from(*consonant).lines()
            }
        }
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

    pub fn nested_radius(&self, consonant_radius: f32) -> f32 {
        consonant_radius * 0.4
    }

    pub fn position_data(
        &self,
        word_radius: f32,
        number_of_letters: usize,
        index: usize,
    ) -> PositionData {
        let distance = match VocalPlacement::from(*self) {
            VocalPlacement::OnLine => word_radius,
            VocalPlacement::Outside => {
                word_radius + self.radius(word_radius, number_of_letters) * 1.5
            }
            VocalPlacement::Inside => {
                if number_of_letters > 1 {
                    word_radius - self.radius(word_radius, number_of_letters) * 1.5
                } else {
                    0.0
                }
            }
        };

        let angle = index as f32 * (360.0 / number_of_letters as f32);

        PositionData {
            distance,
            angle: Angle::new_degree(angle),
            angle_placement: AnglePlacement::Relative,
        }
    }

    pub fn nested_position_data(
        &self,
        consonant_placement: ConsonantPlacement,
        consonant_radius: f32,
        consonant_distance: f32,
        word_radius: f32,
    ) -> PositionData {
        match VocalPlacement::from(*self) {
            VocalPlacement::Inside => PositionData {
                angle: Angle::new_degree(180.0),
                distance: consonant_radius,
                angle_placement: AnglePlacement::Absolute,
            },
            VocalPlacement::Outside => PositionData {
                angle: Angle::new_degree(0.0),
                distance: word_radius + self.nested_radius(consonant_radius) * 1.5,
                angle_placement: AnglePlacement::Absolute,
            },
            VocalPlacement::OnLine => match consonant_placement {
                ConsonantPlacement::ShallowCut => PositionData {
                    angle: Angle::new_degree(0.0),
                    distance: word_radius - consonant_distance,
                    angle_placement: AnglePlacement::Absolute,
                },
                ConsonantPlacement::DeepCut
                | ConsonantPlacement::Inside
                | ConsonantPlacement::OnLine => PositionData {
                    angle: Angle::new_degree(0.0),
                    distance: 0.0,
                    angle_placement: AnglePlacement::Absolute,
                },
            },
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
            Letter::Consonant(consonant) | Letter::ConsonantWithVocal { consonant, .. } => {
                consonant.radius(word_radius, number_of_letters)
            }
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
            Letter::Consonant(consonant) | Letter::ConsonantWithVocal { consonant, .. } => {
                consonant.position_data(word_radius, number_of_letters, index)
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Default, Deref, DerefMut, Component, Reflect)]
#[reflect(Component)]
pub struct NestedLetter(pub Option<Entity>);

#[derive(Bundle)]
pub struct LetterBundle {
    pub letter: Letter,
    pub radius: Radius,
    pub position_data: PositionData,
    pub dots: CircleChildren,
    pub line_slots: LineSlotChildren,
    pub interaction: Interaction,
    pub nested_letter: NestedLetter,
}

impl LetterBundle {
    pub fn new(
        letter: Letter,
        word_radius: f32,
        number_of_letters: usize,
        index: usize,
        nested: Option<Entity>,
    ) -> Self {
        Self {
            letter,
            radius: Radius(letter.radius(word_radius, number_of_letters)),
            position_data: letter.position_data(word_radius, number_of_letters, index),
            dots: Default::default(),
            line_slots: Default::default(),
            interaction: Interaction::default(),
            nested_letter: NestedLetter(nested),
        }
    }
}

#[derive(Debug, Copy, Clone, Default, Component, Reflect)]
#[reflect(Component)]
pub struct NestedVocal;

#[derive(Copy, Clone, Default, Component, Reflect)]
#[reflect(Component)]
pub struct NestedVocalPositionCorrection;

#[derive(Bundle)]
pub struct NestedVocalPositionCorrectionBundle {
    pub nested_vocal_position_correction: NestedVocalPositionCorrection,
    pub spatial_bundle: SpatialBundle,
    pub position_data: PositionData,
}

impl NestedVocalPositionCorrectionBundle {
    pub fn new(consonant_distance: f32) -> Self {
        Self {
            nested_vocal_position_correction: NestedVocalPositionCorrection,
            spatial_bundle: SpatialBundle::VISIBLE_IDENTITY,
            position_data: PositionData {
                angle: Angle::new_degree(0.0),
                distance: -consonant_distance,
                angle_placement: AnglePlacement::Relative,
            },
        }
    }
}

#[derive(Bundle)]
pub struct NestedVocalBundle {
    pub letter_bundle: LetterBundle,
    pub nested_vocal: NestedVocal,
}

impl NestedVocalBundle {
    pub fn new(
        vocal: Vocal,
        consonant_placement: ConsonantPlacement,
        consonant_radius: f32,
        consonant_distance: f32,
        word_radius: f32,
    ) -> Self {
        Self {
            letter_bundle: LetterBundle {
                letter: Letter::Vocal(vocal),
                radius: Radius(vocal.nested_radius(consonant_radius)),
                position_data: vocal.nested_position_data(
                    consonant_placement,
                    consonant_radius,
                    consonant_distance,
                    word_radius,
                ),
                dots: Default::default(),
                line_slots: Default::default(),
                interaction: Interaction::default(),
                nested_letter: NestedLetter::default(),
            },
            nested_vocal: NestedVocal,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ConsonantPlacement {
    DeepCut,
    Inside,
    ShallowCut,
    OnLine,
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

impl VocalDecoration {
    pub fn dots(&self) -> usize {
        0
    }

    pub fn lines(&self) -> usize {
        match self {
            Self::LineOutside | Self::LineInside => 1,
            _ => 0,
        }
    }
}
