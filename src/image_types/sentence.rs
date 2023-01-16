use crate::image_types::{
    new_stroke_mode, AnglePlacement, CircleChildren, LineSlotChildren, PositionData, Radius, Text,
    SVG_SIZE,
};
use crate::math::angle::Degree;
use crate::style::Styles;
use crate::svg_view::Interaction;
use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::DrawMode;
use crate::svg;

pub const OUTER_CIRCLE_SIZE: f32 = 10.0;

#[derive(Debug, Copy, Clone, Default, Component, Reflect)]
#[reflect(Component)]
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
    pub interaction: Interaction,
    pub svg_element: svg::SVGElement
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
            interaction: Interaction::default(),
            svg_element: svg::SVGElement::Group(svg::Group::new())
        }
    }
}

// needed for reflection
pub fn add_shape_for_sentence(
    mut commands: Commands,
    query: Query<Entity, Added<Sentence>>,
    styles: Res<Styles>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(ShapeBundle {
            mode: DrawMode::Stroke(new_stroke_mode(styles.svg_color)),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        });
    }
}
