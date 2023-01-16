use super::consonant::ConsonantPlacement;
use crate::image_types::{
    AnglePlacement, Letter, LetterBundle, NestedLetter, PositionData, Radius, Text,
};
use crate::math::angle::Degree;
use crate::svg;
use crate::svg_view::Interaction;
use bevy::prelude::*;
use strum_macros::EnumIter;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect, FromReflect, EnumIter)]
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
            angle: Degree::new(angle),
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
                angle: Degree::new(180.0),
                distance: consonant_radius,
                angle_placement: AnglePlacement::Absolute,
            },
            VocalPlacement::Outside => PositionData {
                angle: Degree::new(0.0),
                distance: word_radius + self.nested_radius(consonant_radius) * 1.5,
                angle_placement: AnglePlacement::Absolute,
            },
            VocalPlacement::OnLine => match consonant_placement {
                ConsonantPlacement::ShallowCut => PositionData {
                    angle: Degree::new(0.0),
                    distance: word_radius - consonant_distance,
                    angle_placement: AnglePlacement::Absolute,
                },
                ConsonantPlacement::DeepCut
                | ConsonantPlacement::Inside
                | ConsonantPlacement::OnLine => PositionData {
                    angle: Degree::new(0.0),
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
            _ => return Err(format!("'{}' it is not a valid vocal!", value)),
        };

        Ok(vocal)
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
                angle: Degree::new(0.0),
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
        text: String,
        vocal: Vocal,
        consonant_placement: ConsonantPlacement,
        consonant_radius: f32,
        consonant_distance: f32,
        word_radius: f32,
    ) -> Self {
        Self {
            letter_bundle: LetterBundle {
                text: Text(text),
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
                svg_element: svg::SVGElement::Circle(svg::Circle::default()),
            },
            nested_vocal: NestedVocal,
        }
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
