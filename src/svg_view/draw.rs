use crate::image_types::{
    AnglePlacement, CircleChildren, Dot, Letter, LineSlot, NestedVocal,
    NestedVocalPositionCorrection, PositionData, Radius, Sentence, Word, OUTER_CIRCLE_SIZE,
};
use crate::math;
use crate::math::angle::{Angle, Radian};
use crate::math::{Intersection, IntersectionResult};
use crate::svg;
use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_prototype_lyon::prelude::tess::path::path::Builder;
use bevy_prototype_lyon::prelude::*;
use itertools::Itertools;

pub struct DrawPlugin;

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_transform)
            .add_system(
                correct_nested_vocal_with_outside_placement_position.before(update_transform),
            )
            .add_system(draw_sentence.before(draw))
            .add_system(draw_word_and_letter.after(update_transform).before(draw))
            .add_system(draw_nested_vocal.before(draw))
            .add_system(draw_line_slot.after(update_transform).before(draw))
            .add_system(draw_dots)
            .add_system(draw);
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

fn draw(mut query: Query<(&svg::SVGElement, &mut Path), Changed<svg::SVGElement>>) {
    for (svg_element, mut path) in query.iter_mut() {
        let mut path_builder = Builder::new();
        svg_element.add_geometry(&mut path_builder);
        *path = Path(path_builder.build());
    }
}

fn draw_sentence(
    mut query: Query<(&mut svg::SVGElement, &Radius), (Changed<Radius>, With<Sentence>)>,
) {
    for (mut svg_element, radius) in query.iter_mut() {
        debug!("Redraw sentence");
        let radius = **radius;

        let mut group = svg::Group::new();

        let outer_circle = svg::Circle::new(radius + OUTER_CIRCLE_SIZE);
        let inner_circle = svg::Circle::new(radius);

        group.push(outer_circle);
        group.push(inner_circle);

        *svg_element = svg::SVGElement::Group(group);
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
    mut word_query: Query<
        (&Radius, &CircleChildren, &mut svg::SVGElement),
        (With<Word>, Without<Letter>),
    >,
    mut letter_query: Query<
        (
            &Letter,
            &Radius,
            &PositionData,
            &Transform,
            &mut svg::SVGElement,
        ),
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

        let word_circle = math::Circle {
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
            mut letter_svg_element,
        )) = letter_iter.fetch_next()
        {
            debug!("Redraw letter: {:?}", letter);

            if letter.is_cutting() {
                let letter_circle = math::Circle {
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

                    *letter_svg_element =
                        generate_letter_path(**letter_radius, letter_intersections).into();
                } else {
                    error!("{:?} should intersect with word but it doesn't!", letter);
                    *letter_svg_element = svg::Circle::new(**letter_radius).into();
                }
            } else {
                *letter_svg_element = svg::Circle::new(**letter_radius).into();
            }
        }

        *word_svg_element = if word_intersections.is_empty() {
            svg::Circle::new(**word_radius).into()
        } else {
            generate_word_path(**word_radius, word_intersections).into()
        };
    }
}

fn draw_nested_vocal(
    mut query: Query<(&Radius, &mut svg::SVGElement), (With<NestedVocal>, Changed<Radius>)>,
) {
    for (radius, mut svg_element) in query.iter_mut() {
        debug!("Redraw nested vocal");

        *svg_element = svg::Circle::new(**radius).into();
    }
}

fn draw_dots(mut query: Query<(&mut svg::SVGElement, &Radius), (Changed<Radius>, With<Dot>)>) {
    for (mut svg_element, radius) in query.iter_mut() {
        debug!("Redraw dot");

        *svg_element = svg::Circle::new(**radius).into();
    }
}

fn draw_line_slot(
    mut query: Query<(&mut svg::SVGElement, &Transform), (With<LineSlot>, Changed<PositionData>)>,
) {
    for (mut svg_element, transform) in query.iter_mut() {
        debug!("Redraw line_slot");

        *svg_element = svg::Line::new(
            Vec2::ZERO,
            transform.translation.truncate().normalize_or_zero() * 10.0,
        )
        .into();
    }
}

fn sort_intersections_by_angle(c1: math::Circle, c2: math::Circle, a: Vec2, b: Vec2) -> [Vec2; 2] {
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

fn generate_arc_path(radius: f32, [start, end]: [Vec2; 2]) -> svg::Path {
    let start_angle = Radian::angle_from_vec(start).to_degrees().normalize();
    let end_angle = Radian::angle_from_vec(end).to_degrees().normalize();

    let is_large_arc = (end_angle - start_angle).inner().abs() > 180.0;
    let large_arc_flag = !(is_large_arc ^ (start_angle < end_angle));

    let mut path = svg::Path::new();

    path.push(svg::PathElement::MoveTo(Vec2::new(start.x, -start.y)));
    path.push(svg::PathElement::Arc {
        radius,
        large_arc: large_arc_flag,
        end: Vec2::new(end.x, -end.y),
    });

    path
}

fn generate_word_path(word_radius: f32, intersections: Vec<Vec2>) -> svg::Path {
    intersections
        .into_iter()
        .circular_tuple_windows::<(_, _)>()
        .skip(1)
        .step_by(2)
        .flat_map(|(start, end)| generate_arc_path(word_radius, [start, end]).elements)
        .collect::<Vec<_>>()
        .into()
}

fn generate_letter_path(letter_radius: f32, [end, start]: [Vec2; 2]) -> svg::Path {
    generate_arc_path(letter_radius, [start, end])
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::svg::PathElement;

    #[test]
    fn should_not_swap_intersections_for_non_overlapping_origin() {
        let c1 = math::Circle {
            radius: 10.0,
            position: Default::default(),
        };

        let c2 = math::Circle {
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
        let c1 = math::Circle {
            radius: 10.0,
            position: Default::default(),
        };

        let c2 = math::Circle {
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
        let c1 = math::Circle {
            radius: 10.0,
            position: Default::default(),
        };

        let c2 = math::Circle {
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
        let c1 = math::Circle {
            radius: 10.0,
            position: Default::default(),
        };

        let c2 = math::Circle {
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

        let result = generate_arc_path(r, [a, b]).elements[1];

        if let PathElement::Arc { large_arc, .. } = result {
            assert!(!large_arc);
        } else {
            panic!("Wasn't an arc!");
        }
    }

    #[test]
    fn should_set_large_arc_flag_to_zero_for_non_overlapping_origin() {
        let r = 5.0;
        let a = Vec2::new(r, 0.0);
        let b = Vec2::new(-r, -0.5);

        let result = generate_arc_path(r, [a, b]).elements[1];

        if let PathElement::Arc { large_arc, .. } = result {
            assert!(large_arc);
        } else {
            panic!("Wasn't an arc!");
        }
    }

    #[test]
    fn should_not_set_large_arc_flag_to_zero_for_overlapping_origin() {
        let r = 5.0;
        let a = Vec2::new(-r, -0.5);
        let b = Vec2::new(r, -0.5);

        let result = generate_arc_path(r, [a, b]).elements[1];

        if let PathElement::Arc { large_arc, .. } = result {
            assert!(!large_arc);
        } else {
            panic!("Wasn't an arc!");
        }
    }

    #[test]
    fn should_set_large_arc_flag_to_zero_for_overlapping_origin() {
        let r = 5.0;
        let a = Vec2::new(-r, 0.0);
        let b = Vec2::new(0.0, r);

        let result = generate_arc_path(r, [a, b]).elements[1];

        if let PathElement::Arc { large_arc, .. } = result {
            assert!(large_arc);
        } else {
            panic!("Wasn't an arc!");
        }
    }
}
