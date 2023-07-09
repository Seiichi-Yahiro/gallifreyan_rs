use super::{
    AnglePlacement, CircleChildren, LineSlotChildren, PositionData, Radius, Text, SVG_SIZE,
};
use crate::math::angle::Degree;
use bevy::prelude::*;

pub const OUTER_CIRCLE_SIZE: f32 = 10.0;

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct Sentence;

impl Sentence {
    pub fn radius() -> f32 {
        SVG_SIZE * 0.9 / 2.0
    }

    pub fn position_data() -> PositionData {
        PositionData {
            angle: Degree::new(0.0),
            distance: 0.0,
            angle_placement: AnglePlacement::Absolute,
        }
    }
}

#[derive(Bundle)]
pub struct SentenceBundle {
    pub sentence: Sentence,
    pub text: Text,
    pub radius: Radius,
    pub position_data: PositionData,
    pub words: CircleChildren,
    pub line_slots: LineSlotChildren,
}

impl SentenceBundle {
    pub fn new(sentence: String) -> Self {
        Self {
            sentence: Sentence,
            text: Text(sentence),
            radius: Radius(Sentence::radius()),
            position_data: Sentence::position_data(),
            words: CircleChildren::default(),
            line_slots: LineSlotChildren::default(),
        }
    }
}
