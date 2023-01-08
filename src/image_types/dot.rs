use crate::constraints::DistanceConstraints;
use crate::image_types::{new_fill_mode, AnglePlacement, PositionData, Radius};
use crate::math::angle::Degree;
use crate::style::Styles;
use crate::svg_view::Interaction;
use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::DrawMode;

#[derive(Debug, Copy, Clone, Default, Component, Reflect)]
#[reflect(Component)]
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
    pub interaction: Interaction,
    pub distance_constraints: DistanceConstraints,
}

impl DotBundle {
    pub fn new(consonant_radius: f32, number_of_dots: usize, index: usize) -> Self {
        Self {
            dot: Dot,
            radius: Radius(Dot::radius(consonant_radius)),
            position_data: Dot::position_data(consonant_radius, number_of_dots, index),
            interaction: Interaction::default(),
            distance_constraints: DistanceConstraints::default(),
        }
    }
}

// needed for reflection
pub fn add_shape_for_dot(
    mut commands: Commands,
    query: Query<Entity, Added<Dot>>,
    styles: Res<Styles>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(ShapeBundle {
            mode: DrawMode::Fill(new_fill_mode(styles.svg_color)),
            transform: Transform::from_xyz(0.0, 0.0, 0.1),
            ..default()
        });
    }
}
