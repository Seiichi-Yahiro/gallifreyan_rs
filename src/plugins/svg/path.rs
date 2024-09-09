use crate::math::angle::{Angle, Radian};
use crate::math::circle::Circle;
use bevy::math::Vec2;
use bevy_prototype_lyon::prelude::tess::path::Builder;
use bevy_prototype_lyon::prelude::Geometry;
use bevy_prototype_lyon::shapes;
use itertools::Itertools;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone)]
pub struct Path {
    pub elements: Vec<PathElement>,
}

impl From<Vec<PathElement>> for Path {
    fn from(value: Vec<PathElement>) -> Self {
        Self { elements: value }
    }
}

impl Path {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn push(&mut self, element: PathElement) {
        self.elements.push(element);
    }

    pub fn path_string(&self, flip_y_axis: bool) -> String {
        if flip_y_axis {
            self.elements
                .iter()
                .map(|element| match element {
                    PathElement::MoveTo(it) => PathElement::MoveTo(Vec2::new(it.x, -it.y)),
                    PathElement::Arc {
                        end,
                        large_arc,
                        radius,
                    } => PathElement::Arc {
                        end: Vec2::new(end.x, -end.y),
                        large_arc: *large_arc,
                        radius: *radius,
                    },
                })
                .map(|element| element.to_string())
                .join(" ")
        } else {
            self.elements
                .iter()
                .map(|element| element.to_string())
                .join(" ")
        }
    }
}

impl Geometry for Path {
    fn add_geometry(&self, b: &mut Builder) {
        shapes::SvgPathShape {
            svg_doc_size_in_px: Default::default(),
            svg_path_string: self.path_string(false),
        }
        .add_geometry(b);
    }
}

#[derive(Debug, Copy, Clone)]
pub enum PathElement {
    MoveTo(Vec2),
    Arc {
        radius: f32,
        large_arc: bool,
        end: Vec2,
    },
}

impl Display for PathElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PathElement::MoveTo(pos) => {
                write!(f, "M {} {}", pos.x, pos.y)
            }
            PathElement::Arc {
                radius,
                large_arc,
                end,
            } => {
                write!(
                    f,
                    "A {} {} 0 {} 1 {} {}",
                    radius,
                    radius,
                    i32::from(*large_arc),
                    end.x,
                    end.y
                )
            }
        }
    }
}

pub fn sort_intersections_by_angle(c1: Circle, c2: Circle, a: Vec2, b: Vec2) -> [Vec2; 2] {
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

fn generate_arc_path(radius: f32, [start, end]: [Vec2; 2]) -> Path {
    let start_angle = Radian::angle_from_vec(start).to_degrees().normalize();
    let end_angle = Radian::angle_from_vec(end).to_degrees().normalize();

    let is_large_arc = (end_angle - start_angle).inner().abs() > 180.0;
    let large_arc_flag = !(is_large_arc ^ (start_angle < end_angle));

    let mut path = Path::new();

    path.push(PathElement::MoveTo(Vec2::new(start.x, -start.y)));
    path.push(PathElement::Arc {
        radius,
        large_arc: large_arc_flag,
        end: Vec2::new(end.x, -end.y),
    });

    path
}

pub fn generate_word_path(word_radius: f32, intersections: Vec<Vec2>) -> Path {
    intersections
        .into_iter()
        .circular_tuple_windows::<(_, _)>()
        .skip(1)
        .step_by(2)
        .flat_map(|(start, end)| generate_arc_path(word_radius, [start, end]).elements)
        .collect::<Vec<_>>()
        .into()
}

pub fn generate_letter_path(letter_radius: f32, [end, start]: [Vec2; 2]) -> Path {
    generate_arc_path(letter_radius, [start, end])
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
