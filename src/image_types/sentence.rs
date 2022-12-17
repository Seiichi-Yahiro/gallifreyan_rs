use crate::image_types::{
    AnglePlacement, CircleChildren, LineSlotChildren, PositionData, Radius, Text, STROKE_MODE,
    SVG_SIZE,
};
use crate::math::Angle;
use crate::svg_view::Interaction;
use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::DrawMode;

#[derive(Debug, Copy, Clone, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Sentence;

impl Sentence {
    pub fn radius() -> f32 {
        SVG_SIZE * 0.9 / 2.0
    }

    pub fn position_data() -> PositionData {
        PositionData {
            angle: Angle::new_degree(0.0),
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
    pub shape: ShapeBundle,
    pub interaction: Interaction,
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
            shape: ShapeBundle {
                mode: DrawMode::Stroke(STROKE_MODE),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
            interaction: Interaction::default(),
        }
    }
}
