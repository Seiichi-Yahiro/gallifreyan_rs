mod camera;

use bevy::prelude::*;

pub struct SVGViewPlugin;

impl Plugin for SVGViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(camera::CameraPlugin);
    }
}
