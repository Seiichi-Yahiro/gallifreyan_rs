use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

pub struct WorldCursorPlugin;

impl Plugin for WorldCursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldCursor>()
            .add_system(calculate_world_cursor);
    }
}

#[derive(Debug, Default, Resource)]
pub struct WorldCursor {
    pub delta: Vec2,
    pub pos: Vec2,
    pub ndc: Vec2,
}

fn calculate_world_cursor(
    mut world_cursor: ResMut<WorldCursor>,
    windows: Res<Windows>,
    camera_query: Query<(&Camera, &OrthographicProjection, &GlobalTransform)>,
    mut last_cursor_pos: Local<Option<Vec2>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
) {
    if mouse_motion_events.iter().last().is_none() {
        return;
    }

    let window = windows.primary();

    let current_cursor_pos = match window.cursor_position() {
        Some(current_pos) => current_pos,
        None => return,
    };

    if let Ok((camera, projection, global_transform)) = camera_query.get_single() {
        let viewport_size = camera
            .logical_viewport_size()
            .unwrap_or_else(|| Vec2::new(window.width(), window.height()));

        let viewport_pos = camera
            .logical_viewport_rect()
            .map(|(min, _max)| min)
            .unwrap_or(Vec2::ZERO);

        let projection_size = Vec2::new(
            projection.right - projection.left,
            projection.top - projection.bottom,
        ) * projection.scale;

        let world_units_per_device_pixel = projection_size / viewport_size;

        let cursor_delta = current_cursor_pos - last_cursor_pos.unwrap_or(current_cursor_pos);
        world_cursor.delta = cursor_delta * world_units_per_device_pixel;

        let ray = camera.viewport_to_world(global_transform, current_cursor_pos - viewport_pos);

        if let Some(ray) = ray {
            world_cursor.pos = ray.origin.truncate();

            let ndc = camera.world_to_ndc(global_transform, world_cursor.pos.extend(0.0));

            if let Some(ndc) = ndc {
                world_cursor.ndc = ndc.truncate();
            }
        }

        *last_cursor_pos = Some(current_cursor_pos);
    }
}
