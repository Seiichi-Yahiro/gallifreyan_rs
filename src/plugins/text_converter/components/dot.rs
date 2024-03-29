use super::{AnglePlacement, PositionData, Radius};
use crate::math::angle::Degree;
use bevy::prelude::*;

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct Dot;

impl Dot {
    pub fn radius(consonant_radius: f32) -> f32 {
        consonant_radius * 0.1
    }

    pub fn position_data(
        consonant_radius: f32,
        number_of_dots: usize,
        index: usize,
    ) -> PositionData {
        const LETTER_SIDE_ANGLE: f32 = 180.0;
        const DOT_DISTANCE_ANGLE: f32 = 45.0;

        let center_dots_on_letter_side_angle: f32 =
            ((number_of_dots - 1) as f32 * DOT_DISTANCE_ANGLE) / 2.0;

        let distance = consonant_radius - Self::radius(consonant_radius) * 1.5;

        let angle = index as f32 * DOT_DISTANCE_ANGLE - center_dots_on_letter_side_angle
            + LETTER_SIDE_ANGLE;

        PositionData {
            distance,
            angle: Degree::new(angle),
            angle_placement: AnglePlacement::Absolute,
        }
    }
}

#[derive(Bundle)]
pub struct DotBundle {
    pub dot: Dot,
    pub radius: Radius,
    pub position_data: PositionData,
}

impl DotBundle {
    pub fn new(consonant_radius: f32, number_of_dots: usize, index: usize) -> Self {
        Self {
            dot: Dot,
            radius: Radius(Dot::radius(consonant_radius)),
            position_data: Dot::position_data(consonant_radius, number_of_dots, index),
        }
    }
}
