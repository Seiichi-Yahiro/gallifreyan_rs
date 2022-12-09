use crate::image_types::{
    AnglePlacement, CircleChildren, Letter, LineSlot, Placement, PositionData, Radius, Word,
};
use crate::math::{angle_from_position, Circle, Intersection, IntersectionResult};
use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_prototype_lyon::prelude::tess::path::path::Builder;
use bevy_prototype_lyon::prelude::*;
use itertools::Itertools;

pub struct DrawPlugin;

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_position_data)
            .add_system(draw_circle)
            .add_system(draw_word_and_letter.after(update_position_data))
            .add_system(draw_line_slot.after(update_position_data));
    }
}

fn update_position_data(mut query: Query<(&mut Transform, &PositionData), Changed<PositionData>>) {
    for (mut transform, position_data) in query.iter_mut() {
        let translation = Vec3::new(0.0, -position_data.distance, transform.translation.z);
        let rotation = Quat::from_rotation_z(position_data.angle.as_radians());

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

fn draw_circle(
    mut query: Query<(&mut Path, &Radius), (Changed<Radius>, Without<Word>, Without<Letter>)>,
) {
    for (mut path, radius) in query.iter_mut() {
        *path = generate_circle_path(**radius);
    }
}

fn draw_word_and_letter(
    changed_word_query: Query<Entity, (With<Word>, Changed<Radius>)>,
    changed_letter_query: Query<
        &Parent,
        (With<Letter>, Or<(Changed<Radius>, Changed<PositionData>)>),
    >,
    mut word_query: Query<(&Radius, &CircleChildren, &mut Path), (With<Word>, Without<Letter>)>,
    mut letter_query: Query<
        (&Radius, &PositionData, &Transform, &Placement, &mut Path),
        (With<Letter>, Without<Word>),
    >,
) {
    let words: HashSet<Entity> = changed_letter_query
        .iter()
        .map(Parent::get)
        .chain(changed_word_query.iter())
        .collect();

    let mut word_iter = word_query.iter_many_mut(words.iter());

    while let Some((word_radius, letters, mut word_path)) = word_iter.fetch_next() {
        let word_circle = Circle {
            radius: **word_radius,
            position: Vec2::ZERO,
        };

        let mut word_intersections: Vec<Vec2> = Vec::new();

        let mut letter_iter = letter_query.iter_many_mut(letters.iter());

        while let Some((
            letter_radius,
            letter_position_data,
            letter_transform,
            placement,
            mut letter_path,
        )) = letter_iter.fetch_next()
        {
            match placement {
                Placement::DeepCut | Placement::ShallowCut => {
                    let letter_circle = Circle {
                        radius: **letter_radius,
                        position: letter_transform.translation.truncate(),
                    };

                    if let IntersectionResult::Two(a, b) = word_circle.intersection(&letter_circle)
                    {
                        let sorted_intersections =
                            sort_intersections_by_angle(word_circle, letter_circle, a, b);

                        word_intersections.extend(sorted_intersections.iter());

                        let letter_intersections = sorted_intersections
                            .map(|pos| pos - letter_circle.position)
                            .map(|pos| {
                                Vec2::from_angle(-letter_position_data.angle.as_radians())
                                    .rotate(pos)
                            });

                        *letter_path = generate_letter_path(**letter_radius, letter_intersections);
                    }
                }
                _ => {
                    *letter_path = generate_circle_path(**letter_radius);
                }
            }
        }

        *word_path = if word_intersections.is_empty() {
            generate_circle_path(**word_radius)
        } else {
            generate_word_path(**word_radius, word_intersections)
        };
    }
}

fn sort_intersections_by_angle(c1: Circle, c2: Circle, a: Vec2, b: Vec2) -> [Vec2; 2] {
    let angle_a = angle_from_position(a);
    let angle_b = angle_from_position(b);

    let angle_origin = c1.position + Vec2::NEG_Y * c1.radius;
    let distance = c2.position.distance(angle_origin) - c2.radius;
    let is_angle_origin_inside_letter = distance <= 0.0;

    if is_angle_origin_inside_letter ^ (angle_a <= angle_b) {
        [a, b]
    } else {
        [b, a]
    }
}

fn generate_circle_path(radius: f32) -> Path {
    let circle = shapes::Circle {
        radius,
        center: Default::default(),
    };

    generate_path_from_geometry(circle)
}

fn generate_arc_path_string(radius: f32, [start, end]: [Vec2; 2]) -> String {
    let start_angle = angle_from_position(start).as_degrees();
    let end_angle = angle_from_position(end).as_degrees();

    let is_large_arc = (end_angle - start_angle).abs() > 180.0;
    let large_arc_flag = i32::from(!(is_large_arc ^ (start_angle < end_angle)));

    let sweep = 1;

    format!(
        "M {} {} A {} {} 0 {} {} {} {}",
        start.x, -start.y, radius, radius, large_arc_flag, sweep, end.x, -end.y
    )
}

fn generate_word_path(word_radius: f32, intersections: Vec<Vec2>) -> Path {
    let svg_path_string = intersections
        .into_iter()
        .circular_tuple_windows::<(_, _)>()
        .skip(1)
        .step_by(2)
        .map(|(start, end)| generate_arc_path_string(word_radius, [start, end]))
        .join(" ");

    let path_shape = shapes::SvgPathShape {
        svg_doc_size_in_px: Default::default(),
        svg_path_string,
    };

    generate_path_from_geometry(path_shape)
}

fn generate_path_from_geometry(geometry: impl Geometry) -> Path {
    let mut path_builder = Builder::new();
    geometry.add_geometry(&mut path_builder);
    Path(path_builder.build())
}

fn generate_letter_path(letter_radius: f32, [end, start]: [Vec2; 2]) -> Path {
    let svg_path_string = generate_arc_path_string(letter_radius, [start, end]);

    let path_shape = shapes::SvgPathShape {
        svg_doc_size_in_px: Default::default(),
        svg_path_string,
    };

    generate_path_from_geometry(path_shape)
}

fn draw_line_slot(
    mut query: Query<(&mut Path, &Transform), (With<LineSlot>, Changed<PositionData>)>,
) {
    for (mut path, transform) in query.iter_mut() {
        let end = transform.translation.truncate().normalize_or_zero() * 10.0;
        let line = shapes::Line(Vec2::ZERO, end);
        *path = generate_path_from_geometry(line);
    }
}
