use crate::image_types::{
    AnglePlacement, CircleChildren, LineSlotChildren, PositionData, Radius, Text, STROKE_MODE,
};
use crate::math::Angle;
use crate::svg_view::Interaction;
use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::DrawMode;

#[derive(Debug, Copy, Clone, Default, Component, Reflect)]
#[reflect(Component)]
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
            angle: Angle::new_degree(index as f32 * (360.0 / number_of_words as f32)),
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
    pub shape: ShapeBundle,
    pub interaction: Interaction,
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
            shape: ShapeBundle {
                mode: DrawMode::Stroke(STROKE_MODE),
                transform: Transform::from_xyz(0.0, 0.0, 0.1),
                ..default()
            },
            interaction: Interaction::default(),
        }
    }
}
