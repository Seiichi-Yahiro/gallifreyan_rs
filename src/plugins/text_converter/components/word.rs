use super::{AnglePlacement, CircleChildren, LineSlotChildren, PositionData, Radius, Text};
use crate::math::angle::Degree;
use bevy::prelude::*;

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct Word;

impl Word {
    pub fn radius(sentence_radius: f32, number_of_words: usize) -> f32 {
        (sentence_radius * 0.75) / (1.0 + number_of_words as f32 / 2.0)
    }

    pub fn position_data(
        sentence_radius: f32,
        number_of_words: usize,
        index: usize,
    ) -> PositionData {
        PositionData {
            distance: if number_of_words > 1 {
                sentence_radius - Self::radius(sentence_radius, number_of_words) * 1.5
            } else {
                0.0
            },
            angle: Degree::new(index as f32 * (360.0 / number_of_words as f32)),
            angle_placement: AnglePlacement::Absolute,
        }
    }
}

#[derive(Bundle)]
pub struct WordBundle {
    pub word: Word,
    pub text: Text,
    pub radius: Radius,
    pub position_data: PositionData,
    pub letters: CircleChildren,
    pub line_slots: LineSlotChildren,
}

impl WordBundle {
    pub fn new(word: String, sentence_radius: f32, number_of_words: usize, index: usize) -> Self {
        Self {
            word: Word,
            text: Text(word),
            radius: Radius(Word::radius(sentence_radius, number_of_words)),
            position_data: Word::position_data(sentence_radius, number_of_words, index),
            letters: CircleChildren::default(),
            line_slots: LineSlotChildren::default(),
        }
    }
}
