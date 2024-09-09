mod circle;
mod element;
mod group;
mod line;
mod path;

use crate::math::angle::Angle;
use crate::math::circle::Circle as MCircle;
use crate::math::circle::{Intersection, IntersectionResult};
use crate::plugins::svg::element::SVGElement;
use crate::plugins::svg::path::{
    generate_letter_path, generate_word_path, sort_intersections_by_angle,
};
use crate::plugins::text_converter::prelude::*;
use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_prototype_lyon::prelude::tess::path::Builder;
use bevy_prototype_lyon::prelude::*;
use circle::Circle as SVGCircle;

pub const SVG_SIZE: f32 = 1000.0;

#[allow(unused_imports)]
pub mod prelude {
    pub use super::circle::Circle;
    pub use super::element::SVGElement;
    pub use super::group::Group;
    pub use super::line::Line;
    pub use super::path::{Path, PathElement};
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

        let outer_circle = SVGCircle::new(radius.0 + 10.0);
        let inner_circle = SVGCircle::new(radius.0);

        group.push(outer_circle);
        group.push(inner_circle);

        *svg_element = SVGElement::Group(group);
    }
}

fn draw_word_and_letter(
    changed_word_query: Query<Entity, (With<Word>, Changed<Radius>)>,
    changed_letter_query: Query<
        &Parent,
        Or<(Changed<Radius>, Changed<PositionData>, Changed<Letter>)>,
    >,
    mut word_query: Query<
        (&Radius, &CircleChildren, &mut SVGElement),
        (With<Word>, Without<Letter>),
    >,
    mut letter_query: Query<
        (&Letter, &Radius, &PositionData, &Transform, &mut SVGElement),
        Without<Word>,
    >,
) {
    let words: HashSet<Entity> = changed_letter_query
        .iter()
        .map(Parent::get)
        .chain(changed_word_query.iter())
        .collect();

    let mut word_iter = word_query.iter_many_mut(words.iter());

    while let Some((word_radius, letters, mut word_svg_element)) = word_iter.fetch_next() {
        debug!("Redraw word");

        let word_circle = MCircle {
            radius: word_radius.0,
            position: Vec2::ZERO,
        };

        let mut word_intersections: Vec<Vec2> = Vec::new();

        let mut letter_iter = letter_query.iter_many_mut(letters.0.iter());

        while let Some((
            letter,
            letter_radius,
            letter_position_data,
            letter_transform,
            mut letter_svg_element,
        )) = letter_iter.fetch_next()
        {
            debug!("Redraw letter: {:?}", letter);

            if letter.is_cutting() {
                let letter_circle = MCircle {
                    radius: letter_radius.0,
                    position: letter_transform.translation.truncate(),
                };

                if let IntersectionResult::Two(a, b) = word_circle.intersection(&letter_circle) {
                    let sorted_intersections =
                        sort_intersections_by_angle(word_circle, letter_circle, a, b);

                    word_intersections.extend(sorted_intersections.iter());

                    let letter_intersections = sorted_intersections
                        .map(|pos| pos - letter_circle.position)
                        .map(|pos| {
                            Vec2::from_angle(-letter_position_data.angle.to_radians().inner())
                                .rotate(pos)
                        });

                    *letter_svg_element =
                        generate_letter_path(letter_radius.0, letter_intersections).into();
                } else {
                    error!("{:?} should intersect with word but it doesn't!", letter);
                    *letter_svg_element = SVGCircle::new(letter_radius.0).into();
                }
            } else {
                *letter_svg_element = SVGCircle::new(letter_radius.0).into();
            }
        }

        *word_svg_element = if word_intersections.is_empty() {
            SVGCircle::new(word_radius.0).into()
        } else {
            generate_word_path(word_radius.0, word_intersections).into()
        };
    }
}

fn draw_dots(mut query: Query<(&mut SVGElement, &Radius), (Changed<Radius>, With<Dot>)>) {
    for (mut svg_element, radius) in query.iter_mut() {
        debug!("Redraw dot");

        *svg_element = SVGCircle::new(radius.0).into();
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
