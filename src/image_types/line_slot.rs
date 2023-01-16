use crate::image_types::{new_stroke_mode, AnglePlacement, PositionData};
use crate::math::angle::Degree;
use crate::style::Styles;
use crate::svg;
use crate::svg_view::Interaction;
use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::DrawMode;

#[derive(Debug, Copy, Clone, Default, Component, Reflect)]
#[reflect(Component)]
pub struct LineSlot;

impl LineSlot {
    pub fn position_data(
        letter_radius: f32,
        number_of_lines: usize,
        index: usize,
        point_outside: bool,
    ) -> PositionData {
        let letter_side_angle = if point_outside { 0.0 } else { 180.0 };
        const LINE_DISTANCE_ANGLE: f32 = 45.0;
        let center_lines_on_letter_side_angle =
            ((number_of_lines - 1) as f32 * LINE_DISTANCE_ANGLE) / 2.0;

        let distance = letter_radius;

        let angle = index as f32 * LINE_DISTANCE_ANGLE - center_lines_on_letter_side_angle
            + letter_side_angle;

        PositionData {
            distance,
            angle: Degree::new(angle),
            angle_placement: AnglePlacement::Absolute,
        }
    }
}

#[derive(Bundle)]
pub struct LineSlotBundle {
    pub line_slot: LineSlot,
    pub position_data: PositionData,
    pub interaction: Interaction,
    pub svg_element: svg::SVGElement,
}

impl LineSlotBundle {
    pub fn new(
        letter_radius: f32,
        number_of_lines: usize,
        index: usize,
        point_outside: bool,
    ) -> Self {
        Self {
            line_slot: LineSlot,
            position_data: LineSlot::position_data(
                letter_radius,
                number_of_lines,
                index,
                point_outside,
            ),
            interaction: Interaction::default(),
            svg_element: svg::SVGElement::Line(svg::Line::default()),
        }
    }
}

// needed for reflection
pub fn add_shape_for_line_slot(
    mut commands: Commands,
    query: Query<Entity, Added<LineSlot>>,
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
