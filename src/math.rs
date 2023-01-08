pub mod angle;

use bevy::math::{Quat, Vec2};
use bevy::prelude::Reflect;

pub trait Intersection<T> {
    fn intersection(&self, other: &T) -> IntersectionResult;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum IntersectionResult {
    None,
    Infinity,
    One(Vec2),
    Two(Vec2, Vec2),
}

#[derive(Debug, Copy, Clone, PartialEq, Default, Reflect)]
pub struct Circle {
    pub radius: f32,
    pub position: Vec2,
}

impl Intersection<Circle> for Circle {
    ///
    /// The calculation is done in a different coordinate system with the following 2 assumptions:
    /// - Circle a is assumed to be placed at the origin.
    /// - Circle b is assumed to be placed on the x-Axis.
    ///
    ///
    /// x and y are the positions of the intersections
    ///
    /// c is the distance between the circles
    ///
    /// `x² + y² = r_a²`
    /// `(c - x)² + y² = r_b²`
    ///
    /// `x = (r_a² + c² - b²) / 2 * c`
    /// `y_1,2 = +-sqrt(r_a² - x²)`
    ///
    /// Translate x and y back into the original coordinate system
    ///
    /// `unit_x = normalize(b_pos - a_pos)`
    /// `unit_y = rotate(unit_x, 90° counter clockwise)`
    ///
    /// `q1,2 = a_pos + x * unit_x +- y * unit_y`
    ///
    /// Based on http://walter.bislins.ch/blog/index.asp?page=Schnittpunkte+zweier+Kreise+berechnen+%28JavaScript%29
    fn intersection(&self, other: &Circle) -> IntersectionResult {
        let a_to_b = other.position - self.position;
        let distance = a_to_b.length();

        if distance == 0.0 {
            return if self.radius == other.radius {
                IntersectionResult::Infinity
            } else {
                IntersectionResult::None
            };
        }

        let self_radius_squared = self.radius * self.radius;
        let other_radius_squared = other.radius * other.radius;
        let distance_squared = distance * distance;

        let x = (self_radius_squared + distance_squared - other_radius_squared) / (2.0 * distance);

        let determinant = self_radius_squared - x * x;

        if determinant < 0.0 {
            return IntersectionResult::None;
        }

        let y = determinant.sqrt();

        // translate intersection points x and y back into original coordinate system
        let x_unit = a_to_b / distance; // normalize
        let y_unit =
            (Quat::from_rotation_z(std::f32::consts::FRAC_PI_2) * x_unit.extend(0.0)).truncate(); // rotate 90° left

        let x_translation = x_unit * x;
        let y_translation = y_unit * y;

        let q1 = self.position + x_translation + y_translation;

        if determinant == 0.0 {
            IntersectionResult::One(q1)
        } else {
            let q2 = self.position + x_translation - y_translation;
            IntersectionResult::Two(q1, q2)
        }
    }
}

#[cfg(test)]
mod circle_circle_intersection_test {
    use super::*;

    #[test]
    fn should_return_no_circle_intersection_when_same_position_different_radius() {
        let a = Circle {
            radius: 10.0,
            position: Vec2::splat(1.0),
        };

        let b = Circle {
            radius: 5.0,
            position: Vec2::splat(1.0),
        };

        let result = a.intersection(&b);
        let expected = IntersectionResult::None;

        assert_eq!(result, expected);
    }

    #[test]
    fn should_return_infinity_circle_intersection_when_same_position_same_radius() {
        let a = Circle {
            radius: 10.0,
            position: Vec2::splat(1.0),
        };

        let b = Circle {
            radius: 10.0,
            position: Vec2::splat(1.0),
        };

        let result = a.intersection(&b);
        let expected = IntersectionResult::Infinity;

        assert_eq!(result, expected);
    }

    #[test]
    fn should_return_no_circle_intersection() {
        let a = Circle {
            radius: 10.0,
            position: Vec2::splat(100.0),
        };

        let b = Circle {
            radius: 5.0,
            position: Vec2::splat(1.0),
        };

        let result = a.intersection(&b);
        let expected = IntersectionResult::None;

        assert_eq!(result, expected);
    }

    #[test]
    fn should_return_one_circle_intersection() {
        let a = Circle {
            radius: 10.0,
            position: Vec2::new(5.0, 5.0),
        };

        let b = Circle {
            radius: 5.0,
            position: Vec2::new(-10.0, 5.0),
        };

        let result = a.intersection(&b);
        let expected = IntersectionResult::One(Vec2::new(-5.0, 5.0));

        assert_eq!(result, expected);
    }

    #[test]
    fn should_return_two_circle_intersection() {
        let a = Circle {
            radius: 10.0,
            position: Vec2::new(5.0, 5.0),
        };

        let b = Circle {
            radius: 5.0,
            position: Vec2::new(-5.0, 4.0),
        };

        let result = a.intersection(&b);
        let expected = IntersectionResult::Two(
            Vec2::new(-3.232291, -0.6770935),
            Vec2::new(-4.1934524, 8.934519),
        );

        assert_eq!(result, expected);
    }
}
