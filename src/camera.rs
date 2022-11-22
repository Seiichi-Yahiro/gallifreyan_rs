use crate::ui::{is_ui_blocking, ui};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup).add_system_set(
            SystemSet::new()
                .after(ui)
                .with_run_criteria(is_ui_blocking)
                .with_system(camera_pan)
                .with_system(camera_zoom),
        );
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn camera_pan(
    mut camera_query: Query<(&OrthographicProjection, &mut Transform)>,
    mouse_button_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut last_cursor_pos: Local<Option<Vec2>>,
) {
    let window = windows.primary();

    let current_cursor_pos = match window.cursor_position() {
        Some(current_pos) => current_pos,
        None => return,
    };

    let delta_device_pixels = current_cursor_pos - last_cursor_pos.unwrap_or(current_cursor_pos);

    if mouse_button_input.pressed(MouseButton::Left) {
        let (projection, mut transform) = camera_query.single_mut();

        let projection_size = Vec2::new(
            projection.right - projection.left,
            projection.top - projection.bottom,
        ) * projection.scale;

        let window_size = Vec2::new(window.width(), window.height());

        let world_units_per_device_pixel = projection_size / window_size;

        let delta_world = delta_device_pixels * world_units_per_device_pixel;
        let proposed_cam_transform = transform.translation - delta_world.extend(0.0);

        transform.translation = proposed_cam_transform;
    }

    *last_cursor_pos = Some(current_cursor_pos);
}

fn camera_zoom(
    mut camera_query: Query<(&mut OrthographicProjection, &mut Transform)>,
    mut scroll_events: EventReader<MouseWheel>,
    windows: Res<Windows>,
) {
    let pixels_per_line = 100.0;

    let scroll = scroll_events
        .iter()
        .map(|event| match event.unit {
            MouseScrollUnit::Pixel => event.y,
            MouseScrollUnit::Line => event.y * pixels_per_line,
        })
        .sum::<f32>();

    if scroll == 0.0 {
        return;
    }

    let (mut projection, mut transform) = camera_query.single_mut();

    let window = windows.primary();
    let window_size = Vec2::new(window.width(), window.height());

    let mouse_normalized_screen_pos = window
        .cursor_position()
        .map(|cursor_pos| (cursor_pos / window_size) * 2.0 - Vec2::ONE);

    let old_projection_scale = projection.scale;

    projection.scale *= 1.0 - scroll * 0.001;

    if let Some(mouse_normalized_screen_pos) = mouse_normalized_screen_pos {
        let projection_size = Vec2::new(projection.right, projection.top);

        let mouse_world_pos = transform.translation.truncate()
            + mouse_normalized_screen_pos * projection_size * old_projection_scale;

        transform.translation = (mouse_world_pos
            - mouse_normalized_screen_pos * projection_size * projection.scale)
            .extend(transform.translation.z);
    }
}
