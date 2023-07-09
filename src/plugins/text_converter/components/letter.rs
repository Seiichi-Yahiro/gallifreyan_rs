mod consonant;
mod vocal;

use super::{CircleChildren, LineSlotChildren, PositionData, Radius, Text};
use bevy::prelude::*;
use bevy::utils::HashSet;
pub use consonant::*;
pub use vocal::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component)]
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

#[derive(Debug, Copy, Clone, Default, Deref, DerefMut, Component)]
pub struct NestedLetter(pub Option<Entity>);

#[derive(Bundle)]
pub struct LetterBundle {
    pub letter: Letter,
    pub text: Text,
    pub radius: Radius,
    pub position_data: PositionData,
    pub dots: CircleChildren,
    pub line_slots: LineSlotChildren,
    pub nested_letter: NestedLetter,
}

impl LetterBundle {
    pub fn new(
        text: String,
        letter: Letter,
        word_radius: f32,
        number_of_letters: usize,
        index: usize,
    ) -> Self {
        Self {
            letter,
            text: Text(text),
            radius: Radius(letter.radius(word_radius, number_of_letters)),
            position_data: letter.position_data(word_radius, number_of_letters, index),
            dots: Default::default(),
            line_slots: Default::default(),
            nested_letter: NestedLetter::default(),
        }
    }
}

#[derive(Resource)]
pub enum NestingSettings {
    None,
    All,
    Custom(HashSet<(Consonant, Vocal)>),
}

impl NestingSettings {
    pub fn can_nest(&self, consonant: Consonant, vocal: Vocal) -> bool {
        match self {
            NestingSettings::None => false,
            NestingSettings::All => true,
            NestingSettings::Custom(rules) => rules.contains(&(consonant, vocal)),
        }
    }
}
