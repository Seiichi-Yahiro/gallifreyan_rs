use crate::math::angle::Degree;
use bevy::prelude::*;

pub struct AngleConstraintsPlugin;

impl Plugin for AngleConstraintsPlugin {
    fn build(&self, _app: &mut App) {
        // TODO
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct AngleConstraints {
    pub min: Degree,
    pub max: Degree,
}

impl Default for AngleConstraints {
    fn default() -> Self {
        Self {
            min: Degree::new(0.0),
            max: Degree::new(360.0),
        }
    }
}
