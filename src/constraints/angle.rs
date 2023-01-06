use crate::math::Angle;
use bevy::prelude::*;

pub struct AngleConstraintsPlugin;

impl Plugin for AngleConstraintsPlugin {
    fn build(&self, _app: &mut App) {
        // TODO
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct AngleConstraints {
    pub min: Angle,
    pub max: Angle,
}

impl Default for AngleConstraints {
    fn default() -> Self {
        Self {
            min: Angle::new_degree(0.0),
            max: Angle::new_degree(360.0),
        }
    }
}
