mod circle;
mod element;
mod group;
mod line;

use crate::math::angle::Angle;
use crate::plugins::svg::element::SVGElement;
use crate::plugins::text_converter::prelude::*;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::tess::path::Builder;
use bevy_prototype_lyon::prelude::*;

pub const SVG_SIZE: f32 = 1000.0;

pub mod prelude {
    pub use super::circle::Circle;
    pub use super::element::SVGElement;
    pub use super::group::Group;
    pub use super::line::Line;
    pub use super::SVG_SIZE;
}

pub struct SVGPlugin;

impl Plugin for SVGPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                (PrepareLyonSet, PrepareSVGSet).after(TextConversionSet),
                UpdateSVGElementSet,
                UpdateLyonSet,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                add_shape.in_set(PrepareLyonSet),
                add_svg_element.in_set(PrepareSVGSet),
                (
                    update_transform,
                    draw_sentence,
                    draw_word_and_letter,
                    draw_dots,
                    draw_line_slot,
                )
                    .in_set(UpdateSVGElementSet),
                draw.in_set(UpdateLyonSet),
            ),
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct PrepareLyonSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct UpdateLyonSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct PrepareSVGSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct UpdateSVGElementSet;

const STROKE_OPTIONS: StrokeOptions = StrokeOptions::DEFAULT
    .with_line_cap(LineCap::Round)
    .with_line_join(LineJoin::Round)
    .with_line_width(1.0);

fn new_stroke(color: Color) -> Stroke {
    Stroke {
        options: STROKE_OPTIONS,
        color,
    }
}

fn new_fill(color: Color) -> Fill {
    Fill {
        options: FillOptions::DEFAULT,
        color,
    }
}

fn add_shape(
    mut commands: Commands,
    stroke_query: Query<Entity, Or<(Added<Sentence>, Added<Word>, Added<Letter>, Added<LineSlot>)>>,
    fill_query: Query<Entity, Added<Dot>>,
) {
    for entity in stroke_query.iter() {
        debug!("Add stroke shape for {:?}", entity);

        commands
            .entity(entity)
            .insert((ShapeBundle::default(), new_stroke(Color::WHITE)));
    }

    for entity in fill_query.iter() {
        debug!("Add filled shape for {:?}", entity);

        commands
            .entity(entity)
            .insert((ShapeBundle::default(), new_fill(Color::WHITE)));
    }
}

fn add_svg_element(
    mut commands: Commands,
    query: Query<
        Entity,
        (
            Or<(
                Added<Sentence>,
                Added<Word>,
                Added<Letter>,
                Added<Dot>,
                Added<LineSlot>,
            )>,
            Without<SVGElement>,
        ),
    >,
) {
    for entity in query.iter() {
        debug!("Add svg element for {:?}", entity);
        commands.entity(entity).insert(SVGElement::default());
    }
}

fn update_transform(mut query: Query<(&mut Transform, &PositionData), Changed<PositionData>>) {
    for (mut transform, position_data) in query.iter_mut() {
        let translation = Vec3::new(0.0, -position_data.distance, transform.translation.z);
        let rotation = Quat::from_rotation_z(position_data.angle.to_radians().inner());

        match position_data.angle_placement {
            AnglePlacement::Absolute => {
                transform.translation = rotation * translation;
            }
            AnglePlacement::Relative => {
                *transform =
                    Transform::from_rotation(rotation) * Transform::from_translation(translation);
            }
        }
    }
}

fn draw(mut query: Query<(&SVGElement, &mut Path), Changed<SVGElement>>) {
    for (svg_element, mut path) in query.iter_mut() {
        let mut path_builder = Builder::new();
        svg_element.add_geometry(&mut path_builder);
        *path = Path(path_builder.build());
    }
}

fn draw_sentence(mut query: Query<(&mut SVGElement, &Radius), (Changed<Radius>, With<Sentence>)>) {
    for (mut svg_element, radius) in query.iter_mut() {
        debug!("Redraw sentence");
        let mut group = group::Group::new();

        let outer_circle = circle::Circle::new(radius.0 + 10.0);
        let inner_circle = circle::Circle::new(radius.0);

        group.push(outer_circle);
        group.push(inner_circle);

        *svg_element = SVGElement::Group(group);
    }
}

fn draw_word_and_letter(
    mut query: Query<(&mut SVGElement, &Radius), (Changed<Radius>, Or<(With<Word>, With<Letter>)>)>,
) {
    for (mut svg_element, radius) in query.iter_mut() {
        debug!("Redraw word or letter");

        *svg_element = circle::Circle::new(radius.0).into();
    }
}

fn draw_dots(mut query: Query<(&mut SVGElement, &Radius), (Changed<Radius>, With<Dot>)>) {
    for (mut svg_element, radius) in query.iter_mut() {
        debug!("Redraw dot");

        *svg_element = circle::Circle::new(radius.0).into();
    }
}

fn draw_line_slot(
    mut query: Query<(&mut SVGElement, &Transform), (With<LineSlot>, Changed<PositionData>)>,
) {
    for (mut svg_element, transform) in query.iter_mut() {
        debug!("Redraw line_slot");

        *svg_element = line::Line::new(
            Vec2::ZERO,
            transform.translation.truncate().normalize_or_zero() * 10.0,
        )
        .into();
    }
}
