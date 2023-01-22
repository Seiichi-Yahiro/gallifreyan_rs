mod angle;
mod distance;

pub use angle::AngleConstraints;
use bevy::prelude::*;
pub use distance::DistanceConstraints;

pub struct ConstraintsPlugin;

impl Plugin for ConstraintsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(distance::DistanceConstraintsPlugin)
            .add_plugin(angle::AngleConstraintsPlugin);
    }
}
