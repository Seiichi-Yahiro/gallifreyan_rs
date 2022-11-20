use crate::ui::UiState;
use bevy::prelude::*;
use bevy::render::camera::Viewport;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(adjust_camera_viewport);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn adjust_camera_viewport(
    windows: Res<Windows>,
    mut camera_query: Query<&mut Camera>,
    ui_state: Res<UiState>,
) {
    let window = windows.primary();
    let mut camera = camera_query.single_mut();

    let sidebar_width = (ui_state.sidebar_width as f64 * window.scale_factor()) as u32;

    camera.viewport = Some(Viewport {
        physical_position: UVec2::new(sidebar_width, 0),
        physical_size: UVec2::new(
            window.physical_width() - sidebar_width,
            window.physical_height(),
        ),
        ..default()
    });
}
