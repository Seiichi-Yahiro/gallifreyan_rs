use crate::math::Angle;
use bevy::prelude::*;

pub struct AngleConstraintsPlugin;

impl Plugin for AngleConstraintsPlugin {
    fn build(&self, _app: &mut App) {
        // TODO
    }
}

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct AngleConstraints {
    pub min: Angle,
    pub max: Angle,
}
