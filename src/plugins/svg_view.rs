use crate::plugins::text_converter::components::SVG_SIZE;
use crate::plugins::ui::UiBaseSet;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::render::camera::Viewport;
use bevy_egui::{egui, EguiContexts};

pub struct SVGViewPlugin;

impl Plugin for SVGViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<ViewMode>()
            .add_event::<CenterView>()
            .init_resource::<WorldCursor>()
            .add_startup_system(setup)
            .add_systems(
                (adjust_view_port, calculate_world_cursor)
                    .chain()
                    .after(UiBaseSet),
            )
            .add_systems(
                (
                    camera_pan.in_set(OnUpdate(ViewMode::Pan)),
                    camera_zoom,
                    center_view,
                )
                    .after(calculate_world_cursor),
            );
    }
}

#[derive(States, Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub enum ViewMode {
    #[default]
    Select,
    Pan,
}

#[derive(Component)]
pub struct SVGViewCamera;

#[derive(Default)]
pub struct CenterView;

fn setup(mut commands: Commands, mut center_view_events: EventWriter<CenterView>) {
    commands.spawn((Camera2dBundle::default(), SVGViewCamera));
    center_view_events.send_default();
}

#[derive(Deref, DerefMut)]
struct AvailableRect(egui::Rect);

impl Default for AvailableRect {
    fn default() -> Self {
        Self(egui::Rect::NOTHING)
    }
}

fn adjust_view_port(
    egui_contexts: EguiContexts,
    windows: Query<&Window>,
    mut camera_query: Query<&mut Camera, With<SVGViewCamera>>,
    mut available_rect: Local<AvailableRect>,
) {
    let new_rect = egui_contexts.ctx().available_rect();

    if **available_rect != new_rect {
        **available_rect = new_rect;

        let window = windows
            .get_single()
            .expect("There should only be one window!");
        let scale_factor = window.scale_factor();

        let mut camera = camera_query.single_mut();

        let physical_position = UVec2::new(
            (new_rect.left() as f64 * scale_factor) as u32,
            (new_rect.top() as f64 * scale_factor) as u32,
        );

        let physical_size = UVec2::new(
            (new_rect.width() as f64 * scale_factor) as u32,
            (new_rect.height() as f64 * scale_factor) as u32,
        );

        debug!("Update physical viewport position: {:?}", physical_position);
        debug!("Update physical viewport size: {:?}", physical_size);

        camera.viewport = Some(Viewport {
            physical_position,
            physical_size,
            ..default()
        });
    }
}

#[derive(Debug, Default, Resource)]
pub struct WorldCursor {
    pub delta: Vec2,
    pub pos: Vec2,
}

fn calculate_world_cursor(
    mut world_cursor: ResMut<WorldCursor>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &OrthographicProjection, &GlobalTransform)>,
    mut last_cursor_pos: Local<Option<Vec2>>,
) {
    let window = windows
        .get_single()
        .expect("There should only be one window!");

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

        // projection size already contains the projection scale factor
        // projection size represents the area of the world currently visible in world units
        let world_units_per_device_pixel = projection.area.size() / viewport_size;

        let cursor_delta = current_cursor_pos - last_cursor_pos.unwrap_or(current_cursor_pos);
        world_cursor.delta = cursor_delta * world_units_per_device_pixel;

        let ray = camera.viewport_to_world(
            global_transform,
            current_cursor_pos - Vec2::new(viewport_pos.x, 0.0), // TODO find out why only x seems to be necessary?
        );

        if let Some(ray) = ray {
            world_cursor.pos = ray.origin.truncate();
        }
    }

    *last_cursor_pos = Some(current_cursor_pos);
}

fn camera_pan(
    mut camera_query: Query<&mut Transform, With<SVGViewCamera>>,
    world_cursor: Res<WorldCursor>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut is_panning: Local<bool>,
    egui_contexts: EguiContexts,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let ctx = egui_contexts.ctx();
        *is_panning =
            !(ctx.is_pointer_over_area() || ctx.is_using_pointer() || ctx.wants_keyboard_input());
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        *is_panning = false;
    }

    if !*is_panning {
        return;
    }

    if let Ok(mut transform) = camera_query.get_single_mut() {
        transform.translation -= world_cursor.delta.extend(0.0);
    }
}

fn camera_zoom(
    world_cursor: Res<WorldCursor>,
    mut camera_query: Query<
        (
            &Camera,
            &mut OrthographicProjection,
            &mut Transform,
            &GlobalTransform,
        ),
        With<SVGViewCamera>,
    >,
    mut scroll_events: EventReader<MouseWheel>,
    egui_contexts: EguiContexts,
) {
    if egui_contexts.ctx().is_pointer_over_area() {
        return;
    }

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

    if let Ok((camera, mut projection, mut transform, global_transform)) =
        camera_query.get_single_mut()
    {
        // gets the relative (-1 to 1) cursor position on the viewport
        let ndc = camera.world_to_ndc(global_transform, world_cursor.pos.extend(0.0));

        if let Some(ndc) = ndc {
            let scale_factor = 1.0 - scroll * 0.001;
            let scaled_projection_size = projection.area.half_size() * scale_factor;
            let relative_cursor_pos_in_scaled_area = ndc.truncate() * scaled_projection_size;

            // the cursor position needs to be in the same spot after zooming
            transform.translation = (world_cursor.pos - relative_cursor_pos_in_scaled_area)
                .extend(transform.translation.z);

            projection.scale *= scale_factor;
        }
    }
}

fn center_view(
    mut events: EventReader<CenterView>,
    mut camera_query: Query<
        (&Camera, &mut OrthographicProjection, &mut Transform),
        With<SVGViewCamera>,
    >,
    windows: Query<&Window>,
) {
    if events.iter().last().is_some() {
        let (camera, mut orthographic_projection, mut transform) = camera_query.single_mut();

        let viewport_size = camera.logical_viewport_size().unwrap_or_else(|| {
            let window = windows
                .get_single()
                .expect("There should only be one window!");
            Vec2::new(window.width(), window.height())
        });

        orthographic_projection.scale = SVG_SIZE / viewport_size.min_element();
        transform.translation = Vec3::new(0.0, 0.0, transform.translation.z);
    }
}
