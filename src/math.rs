use bevy::math::Vec2;
use std::fmt;

pub trait Angle: Copy + PartialEq + PartialOrd + From<f32> {
    fn inner(self) -> f32;

    fn to_degrees(self) -> Degree;

    fn to_radians(self) -> Radian;

    fn abs(&self) -> Self {
        Self::from(self.inner().abs())
    }

    fn min(&self, other: Self) -> Self {
        Self::from(self.inner().min(other.inner()))
    }

    fn max(&self, other: Self) -> Self {
        Self::from(self.inner().max(other.inner()))
    }

    fn clamp(self, min: Self, max: Self) -> Self;
}

#[derive(Default, Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Degree(pub f32);

impl fmt::Display for Degree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}°", self.0)
    }
}

impl From<Radian> for Degree {
    fn from(value: Radian) -> Self {
        Self(value.0.to_degrees())
    }
}

impl From<f32> for Degree {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl Degree {
    pub fn normalize(self) -> Self {
        Self((self.0 % 360.0 + 360.0) % 360.0)
    }
}

impl Angle for Degree {
    fn inner(self) -> f32 {
        self.0
    }

    fn to_degrees(self) -> Degree {
        self
    }

    fn to_radians(self) -> Radian {
        Radian::from(self)
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        const MIN: Degree = Degree(0.0);
        const MAX: Degree = Degree(360.0);

        let angle = self;

        assert!(
            (MIN..=MAX).contains(&angle),
            "Angle needs to be between 0 and 360 degrees but was {}",
            angle
        );
        assert!(
            (MIN..=MAX).contains(&min),
            "Min needs to be between 0 and 360 degrees but was {}",
            min
        );
        assert!(
            (MIN..=MAX).contains(&max),
            "Max needs to be between 0 and 360 degrees but was {}",
            max
        );

        if min > max && ((angle >= min && angle < MAX) || (angle >= MIN && angle <= max)) {
            angle
        } else if angle < min || angle > max {
            let min_diff = (angle - min).abs();
            let min_distance = min_diff.min(MAX - min_diff);

            let max_diff = (angle - max).abs();
            let max_distance = max_diff.min(MAX - max_diff);

            if min_distance < max_distance {
                min
            } else {
                max
            }
        } else {
            angle
        }
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Radian(pub f32);

impl fmt::Display for Radian {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} rad", self.0)
    }
}

impl From<Degree> for Radian {
    fn from(value: Degree) -> Self {
        Self(value.0.to_radians())
    }
}

impl From<f32> for Radian {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl Radian {
    pub fn angle_from_vec(vec: Vec2) -> Self {
        Radian(-vec.angle_between(Vec2::NEG_Y))
    }
}

impl Angle for Radian {
    fn inner(self) -> f32 {
        self.0
    }

    fn to_degrees(self) -> Degree {
        Degree::from(self)
    }

    fn to_radians(self) -> Radian {
        self
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        self.to_degrees()
            .clamp(min.to_degrees(), max.to_degrees())
            .to_radians()
    }
}

macro_rules! ops {
    ($angle:ty) => {
        impl std::ops::Add<$angle> for $angle {
            type Output = Self;

            fn add(self, rhs: $angle) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl std::ops::Sub<$angle> for $angle {
            type Output = Self;

            fn sub(self, rhs: $angle) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }
    };
}

ops!(Degree);
ops!(Radian);

#[cfg(test)]
mod test {
    use super::*;
    use bevy_prototype_lyon::prelude::tess::geom::euclid::approxeq::ApproxEq;

    #[test]
    fn should_clamp_angles_closer_to_max() {
        let result = Degree(90.0).clamp(Degree(270.0), Degree(360.0));
        assert_eq!(result, Degree(360.0));
    }

    #[test]
    fn should_clamp_angles_closer_to_min() {
        let result = Degree(270.0).clamp(Degree(0.0), Degree(90.0));
        assert_eq!(result, Degree(0.0));
    }

    #[test]
    fn should_not_clamp_angles_in_range() {
        let result = Degree(180.0).clamp(Degree(90.0), Degree(270.0));
        assert_eq!(result, Degree(180.0));
    }

    #[test]
    fn should_clamp_angles_closer_to_min_when_min_greater_than_max() {
        let result = Degree(260.0).clamp(Degree(270.0), Degree(90.0));
        assert_eq!(result, Degree(270.0));
    }

    #[test]
    fn should_clamp_angles_closer_to_max_when_min_greater_than_max() {
        let result = Degree(100.0).clamp(Degree(270.0), Degree(90.0));
        assert_eq!(result, Degree(90.0));
    }

    #[test]
    fn should_not_clamp_angles_in_range_when_min_greater_than_max_1() {
        let result = Degree(10.0).clamp(Degree(270.0), Degree(90.0));
        assert_eq!(result, Degree(10.0));
    }

    #[test]
    fn should_not_clamp_angles_in_range_when_min_greater_than_max_2() {
        let result = Degree(350.0).clamp(Degree(270.0), Degree(90.0));
        assert_eq!(result, Degree(350.0));
    }

    #[test]
    fn should_normalize_negative_degrees_less_than_360() {
        let result = Degree(-180.0).normalize().inner();
        let expected = 180.0;

        assert_eq!(result, expected);
    }

    #[test]
    fn should_normalize_negative_degrees_greater_than_360() {
        let result = Degree(-(360.0 + 180.0)).normalize().inner();
        let expected = 180.0;

        assert_eq!(result, expected);
    }

    #[test]
    fn should_normalize_positive_degrees_less_than_360() {
        let result = Degree(180.0).normalize().inner();
        let expected = 180.0;

        assert_eq!(result, expected);
    }

    #[test]
    fn should_normalize_positive_degrees_greater_than_360() {
        let result = Degree(360.0 + 180.0).normalize().inner();
        let expected = 180.0;

        assert_eq!(result, expected);
    }

    #[test]
    fn should_calculate_angle_from_vec_x() {
        let result = Radian::angle_from_vec(Vec2::X).inner();
        let expected = std::f32::consts::FRAC_PI_2;

        assert!(
            result.approx_eq(&expected),
            "Expected {} but got {}",
            expected,
            result
        );
    }

    #[test]
    fn should_calculate_angle_from_vec_neg_x() {
        let result = Radian::angle_from_vec(Vec2::NEG_X).inner();
        let expected = -std::f32::consts::FRAC_PI_2;

        assert!(
            result.approx_eq(&expected),
            "Expected {} but got {}",
            expected,
            result
        );
    }

    #[test]
    fn should_calculate_angle_from_vec_y() {
        let result = Radian::angle_from_vec(Vec2::Y).inner();
        let expected = std::f32::consts::PI;

        assert!(
            result.approx_eq(&expected),
            "Expected {} but got {}",
            expected,
            result
        );
    }
}
