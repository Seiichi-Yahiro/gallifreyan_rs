use crate::image_types::{
    AnglePlacement, CircleChildren, Dot, Letter, LineSlot, NestedVocal,
    NestedVocalPositionCorrection, PositionData, Radius, Sentence, Word, OUTER_CIRCLE_SIZE,
};
use crate::math::angle::{Angle, Radian};
use crate::math::{Circle, Intersection, IntersectionResult};
use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_prototype_lyon::prelude::tess::path::path::Builder;
use bevy_prototype_lyon::prelude::*;
use itertools::Itertools;

pub struct DrawPlugin;

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_position_data)
            .add_system(
                correct_nested_vocal_with_outside_placement_position.before(update_position_data),
            )
            .add_system(draw_sentence)
            .add_system(draw_word_and_letter.after(update_position_data))
            .add_system(draw_nested_vocal)
            .add_system(draw_line_slot.after(update_position_data))
            .add_system(draw_dots);
    }
}

fn update_position_data(mut query: Query<(&mut Transform, &PositionData), Changed<PositionData>>) {
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

fn correct_nested_vocal_with_outside_placement_position(
    mut position_correction_query: Query<
        (&Parent, &mut PositionData),
        With<NestedVocalPositionCorrection>,
    >,
    parent_query: Query<&PositionData, (Without<NestedVocalPositionCorrection>, With<Letter>)>,
) {
    for (parent, mut position_data) in position_correction_query.iter_mut() {
        if let Ok(parent_position_data) = parent_query.get(parent.get()) {
            position_data.distance = -parent_position_data.distance;
        }
    }
}

fn draw_sentence(mut query: Query<(&mut Path, &Radius), (Changed<Radius>, With<Sentence>)>) {
    for (mut path, radius) in query.iter_mut() {
        debug!("Redraw sentence");
        let radius = **radius;

        let outer_circle = shapes::Circle {
            radius: radius + OUTER_CIRCLE_SIZE,
            center: Default::default(),
        };

        let inner_circle = shapes::Circle {
            radius,
            center: Default::default(),
        };

        let mut path_builder = Builder::new();
        outer_circle.add_geometry(&mut path_builder);
        inner_circle.add_geometry(&mut path_builder);
        *path = Path(path_builder.build());
    }
}

fn draw_word_and_letter(
    changed_word_query: Query<Entity, (With<Word>, Changed<Radius>)>,
    changed_letter_query: Query<
        &Parent,
        (
            Or<(Changed<Radius>, Changed<PositionData>, Changed<Letter>)>,
            Without<NestedVocal>,
        ),
    >,
    mut word_query: Query<(&Radius, &CircleChildren, &mut Path), (With<Word>, Without<Letter>)>,
    mut letter_query: Query<
        (&Letter, &Radius, &PositionData, &Transform, &mut Path),
        Without<Word>,
    >,
) {
    let words: HashSet<Entity> = changed_letter_query
        .iter()
        .map(Parent::get)
        .chain(changed_word_query.iter())
        .collect();

    let mut word_iter = word_query.iter_many_mut(words.iter());

    while let Some((word_radius, letters, mut word_path)) = word_iter.fetch_next() {
        debug!("Redraw word");

        let word_circle = Circle {
            radius: **word_radius,
            position: Vec2::ZERO,
        };

        let mut word_intersections: Vec<Vec2> = Vec::new();

        let mut letter_iter = letter_query.iter_many_mut(letters.iter());

        while let Some((
            letter,
            letter_radius,
            letter_position_data,
            letter_transform,
            mut letter_path,
        )) = letter_iter.fetch_next()
        {
            debug!("Redraw letter: {:?}", letter);

            if letter.is_cutting() {
                let letter_circle = Circle {
                    radius: **letter_radius,
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

                    *letter_path = generate_letter_path(**letter_radius, letter_intersections);
                } else {
                    error!("{:?} should intersect with word but it doesn't!", letter);
                    *letter_path = generate_circle_path(**letter_radius);
                }
            } else {
                *letter_path = generate_circle_path(**letter_radius);
            }
        }

        *word_path = if word_intersections.is_empty() {
            generate_circle_path(**word_radius)
        } else {
            generate_word_path(**word_radius, word_intersections)
        };
    }
}

fn draw_nested_vocal(mut query: Query<(&Radius, &mut Path), (With<NestedVocal>, Changed<Radius>)>) {
    for (radius, mut path) in query.iter_mut() {
        debug!("Redraw nested vocal");
        *path = generate_circle_path(**radius);
    }
}

fn sort_intersections_by_angle(c1: Circle, c2: Circle, a: Vec2, b: Vec2) -> [Vec2; 2] {
    let angle_a = Radian::angle_from_vec(a).to_degrees().normalize();
    let angle_b = Radian::angle_from_vec(b).to_degrees().normalize();

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
    let start_angle = Radian::angle_from_vec(start).to_degrees().normalize();
    let end_angle = Radian::angle_from_vec(end).to_degrees().normalize();

    let is_large_arc = (end_angle - start_angle).inner().abs() > 180.0;
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

fn draw_dots(mut query: Query<(&mut Path, &Radius), (Changed<Radius>, With<Dot>)>) {
    for (mut path, radius) in query.iter_mut() {
        debug!("Redraw dot");
        *path = generate_circle_path(**radius);
    }
}

fn draw_line_slot(
    mut query: Query<(&mut Path, &Transform), (With<LineSlot>, Changed<PositionData>)>,
) {
    for (mut path, transform) in query.iter_mut() {
        debug!("Redraw line_slot");
        let end = transform.translation.truncate().normalize_or_zero() * 10.0;
        let line = shapes::Line(Vec2::ZERO, end);
        *path = generate_path_from_geometry(line);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_not_swap_intersections_for_non_overlapping_origin() {
        let c1 = Circle {
            radius: 10.0,
            position: Default::default(),
        };

        let c2 = Circle {
            radius: 5.0,
            position: Vec2::new(c1.radius, 0.0),
        };

        let a = Vec2::new(8.75, -4.8412285);
        let b = Vec2::new(8.75, 4.8412285);

        let result = sort_intersections_by_angle(c1, c2, a, b);

        assert_eq!(result, [a, b]);
    }

    #[test]
    fn should_swap_intersections_for_non_overlapping_origin() {
        let c1 = Circle {
            radius: 10.0,
            position: Default::default(),
        };

        let c2 = Circle {
            radius: 5.0,
            position: Vec2::new(c1.radius, 0.0),
        };

        let a = Vec2::new(8.75, 4.8412285);
        let b = Vec2::new(8.75, -4.8412285);

        let result = sort_intersections_by_angle(c1, c2, a, b);

        assert_eq!(result, [b, a]);
    }

    #[test]
    fn should_not_swap_intersections_for_overlapping_origin() {
        let c1 = Circle {
            radius: 10.0,
            position: Default::default(),
        };

        let c2 = Circle {
            radius: 5.0,
            position: Vec2::new(0.0, -c1.radius),
        };

        let a = Vec2::new(-4.8412285, -8.75);
        let b = Vec2::new(4.8412285, -8.75);

        let result = sort_intersections_by_angle(c1, c2, a, b);

        assert_eq!(result, [a, b]);
    }

    #[test]
    fn should_swap_intersections_for_overlapping_origin() {
        let c1 = Circle {
            radius: 10.0,
            position: Default::default(),
        };

        let c2 = Circle {
            radius: 5.0,
            position: Vec2::new(0.0, -c1.radius),
        };

        let a = Vec2::new(4.8412285, -8.75);
        let b = Vec2::new(-4.8412285, -8.75);

        let result = sort_intersections_by_angle(c1, c2, a, b);

        assert_eq!(result, [b, a]);
    }

    #[test]
    fn should_not_set_large_arc_flag_to_zero_for_non_overlapping_origin() {
        let r = 5.0;
        let a = Vec2::new(r, 0.0);
        let b = Vec2::new(0.0, r);

        let result = generate_arc_path_string(r, [a, b]);
        let expected = "M 5 -0 A 5 5 0 0 1 0 -5";

        assert_eq!(result, expected);
    }

    #[test]
    fn should_set_large_arc_flag_to_zero_for_non_overlapping_origin() {
        let r = 5.0;
        let a = Vec2::new(r, 0.0);
        let b = Vec2::new(-r, -0.5);

        let result = generate_arc_path_string(r, [a, b]);
        let expected = "M 5 -0 A 5 5 0 1 1 -5 0.5";

        assert_eq!(result, expected);
    }

    #[test]
    fn should_not_set_large_arc_flag_to_zero_for_overlapping_origin() {
        let r = 5.0;
        let a = Vec2::new(-r, -0.5);
        let b = Vec2::new(r, -0.5);

        let result = generate_arc_path_string(r, [a, b]);
        let expected = "M -5 0.5 A 5 5 0 0 1 5 0.5";

        assert_eq!(result, expected);
    }

    #[test]
    fn should_set_large_arc_flag_to_zero_for_overlapping_origin() {
        let r = 5.0;
        let a = Vec2::new(-r, 0.0);
        let b = Vec2::new(0.0, r);

        let result = generate_arc_path_string(r, [a, b]);
        let expected = "M -5 -0 A 5 5 0 1 1 0 -5";

        assert_eq!(result, expected);
    }
}
