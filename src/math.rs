use bevy::math::{Quat, Vec2};
use std::cmp::Ordering;
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

pub fn clamp_angle(angle: f32, min: f32, max: f32) -> f32 {
    if min > max && ((angle >= min && angle < 360.0) || (angle >= 0.0 && angle <= max)) {
        angle
    } else if angle < min || angle > max {
        let min_diff = (angle - min).abs();
        let min_distance = min_diff.min(360.0 - min_diff);

        let max_diff = (angle - max).abs();
        let max_distance = max_diff.min(360.0 - max_diff);

        if min_distance < max_distance {
            min
        } else {
            max
        }
    } else {
        angle
    }
}

#[derive(Default, Debug, Copy, Clone, Reflect)]
pub struct Angle(f32);

impl Angle {
    pub fn new_degree(angle: f32) -> Self {
        Self(adjust_angle(angle))
    }

    pub fn new_radian(angle: f32) -> Self {
        Self(adjust_angle(angle.to_degrees()))
    }

    pub fn as_radians(self) -> f32 {
        self.0.to_radians()
    }

    pub fn as_degrees(self) -> f32 {
        self.0
    }
}

impl PartialEq for Angle {
    fn eq(&self, other: &Self) -> bool {
        self.as_degrees() == other.as_degrees()
    }
}

impl PartialOrd for Angle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_degrees().partial_cmp(&other.as_degrees())
    }
}

pub fn angle_from_position(position: Vec2) -> Angle {
    Angle::new_radian(-position.angle_between(Vec2::NEG_Y))
}

fn adjust_angle(mut angle: f32) -> f32 {
    while angle > 360.0 {
        angle -= 360.0;
    }

    while angle < 0.0 {
        angle += 360.0;
    }

    angle
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

#[cfg(test)]
mod angle_test {
    use super::*;

    #[test]
    fn should_clamp_angles_closer_to_max() {
        let result = clamp_angle(90.0, 270.0, 360.0);
        assert_eq!(result, 360.0);
    }

    #[test]
    fn should_clamp_angles_closer_to_min() {
        let result = clamp_angle(270.0, 0.0, 90.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn should_not_clamp_angles_in_range() {
        let result = clamp_angle(180.0, 90.0, 270.0);
        assert_eq!(result, 180.0);
    }

    #[test]
    fn should_clamp_angles_closer_to_min_when_min_greater_than_max() {
        let result = clamp_angle(260.0, 270.0, 90.0);
        assert_eq!(result, 270.0);
    }

    #[test]
    fn should_clamp_angles_closer_to_max_when_min_greater_than_max() {
        let result = clamp_angle(100.0, 270.0, 90.0);
        assert_eq!(result, 90.0);
    }

    #[test]
    fn should_not_clamp_angles_in_range_when_min_greater_than_max_1() {
        let result = clamp_angle(10.0, 270.0, 90.0);
        assert_eq!(result, 10.0);
    }

    #[test]
    fn should_not_clamp_angles_in_range_when_min_greater_than_max_2() {
        let result = clamp_angle(350.0, 270.0, 90.0);
        assert_eq!(result, 350.0);
    }
}
