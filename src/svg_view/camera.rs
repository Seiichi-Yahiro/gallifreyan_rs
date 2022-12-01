use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::render::camera::Viewport;
use bevy_egui::egui::epaint::Shadow;
use bevy_egui::{egui, EguiContext};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ViewMode::Select)
            .add_startup_system(setup)
            .add_system(ui.after(crate::sidebar::UiSystemLabel))
            .add_system(adjust_view_port.after(crate::sidebar::UiSystemLabel))
            .add_system_set(
                SystemSet::new()
                    .after(ui)
                    .after(adjust_view_port)
                    .with_system(camera_pan)
                    .with_system(camera_zoom),
            );
    }
}

#[derive(Component)]
struct SVGViewCamera;

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), SVGViewCamera));
}

#[derive(Deref, DerefMut)]
struct AvailableRect(egui::Rect);

impl Default for AvailableRect {
    fn default() -> Self {
        Self(egui::Rect::NOTHING)
    }
}

fn adjust_view_port(
    mut egui_context: ResMut<EguiContext>,
    windows: Res<Windows>,
    mut camera_query: Query<&mut Camera, With<SVGViewCamera>>,
    mut available_rect: Local<AvailableRect>,
) {
    let new_rect = egui_context.ctx_mut().available_rect();

    if **available_rect != new_rect {
        **available_rect = new_rect;

        let window = windows.primary();
        let scale_factor = window.scale_factor();

        let mut camera = camera_query.single_mut();

        camera.viewport = Some(Viewport {
            physical_position: UVec2::new(
                (new_rect.left() as f64 * scale_factor) as u32,
                (new_rect.top() as f64 * scale_factor) as u32,
            ),
            physical_size: UVec2::new(
                (new_rect.width() as f64 * scale_factor) as u32,
                (new_rect.height() as f64 * scale_factor) as u32,
            ),
            ..default()
        });
    }
}

fn camera_pan(
    mut camera_query: Query<
        (&Camera, &OrthographicProjection, &mut Transform),
        With<SVGViewCamera>,
    >,
    mouse_button_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut last_cursor_pos: Local<Option<Vec2>>,
    mut is_panning: Local<bool>,
    view_mode: Res<ViewMode>,
    mut egui_context: ResMut<EguiContext>,
) {
    if *view_mode == ViewMode::Select {
        return;
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        let ctx = egui_context.ctx_mut();
        *is_panning =
            !(ctx.is_pointer_over_area() || ctx.is_using_pointer() || ctx.wants_keyboard_input());
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        *is_panning = false;
    }

    let window = windows.primary();

    let current_cursor_pos = match window.cursor_position() {
        Some(current_pos) => current_pos,
        None => return,
    };

    if *is_panning {
        let (camera, projection, mut transform) = camera_query.single_mut();

        let projection_size = Vec2::new(
            projection.right - projection.left,
            projection.top - projection.bottom,
        ) * projection.scale;

        let viewport_size = camera
            .logical_viewport_size()
            .unwrap_or_else(|| Vec2::new(window.width(), window.height()));

        let world_units_per_device_pixel = projection_size / viewport_size;
        let delta_device_pixels =
            current_cursor_pos - last_cursor_pos.unwrap_or(current_cursor_pos);
        let delta_world = delta_device_pixels * world_units_per_device_pixel;
        let proposed_cam_transform = transform.translation - delta_world.extend(0.0);

        transform.translation = proposed_cam_transform;
    }

    *last_cursor_pos = Some(current_cursor_pos);
}

fn camera_zoom(
    mut camera_query: Query<
        (&Camera, &mut OrthographicProjection, &mut Transform),
        With<SVGViewCamera>,
    >,
    mut scroll_events: EventReader<MouseWheel>,
    windows: Res<Windows>,
    mut egui_context: ResMut<EguiContext>,
) {
    let ctx = egui_context.ctx_mut();

    if ctx.is_pointer_over_area() {
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

    let (camera, mut projection, mut transform) = camera_query.single_mut();

    let window = windows.primary();

    let viewport_size = camera
        .logical_viewport_size()
        .unwrap_or_else(|| Vec2::new(window.width(), window.height()));

    let viewport_pos = camera
        .logical_viewport_rect()
        .map(|(min, _max)| min)
        .unwrap_or(Vec2::ZERO);

    let mouse_normalized_screen_pos = window
        .cursor_position()
        .map(|cursor_pos| ((cursor_pos - viewport_pos) / viewport_size) * 2.0 - Vec2::ONE);

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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Resource)]
pub enum ViewMode {
    Select,
    Pan,
}

fn ui(
    mut egui_context: ResMut<EguiContext>,
    mut view_mode: ResMut<ViewMode>,
    mut camera_query: Query<(&mut OrthographicProjection, &mut Transform), With<SVGViewCamera>>,
) {
    egui::Window::new("svg controls")
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .frame(
            egui::Frame::window(&egui_context.ctx_mut().style()).shadow(Shadow {
                extrusion: 0.0,
                color: egui::Color32::BLACK,
            }),
        )
        .fixed_size(egui::Vec2::new(20.0, 60.0))
        .anchor(egui::Align2::LEFT_TOP, egui::Vec2::splat(5.0))
        .show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered_justified(|ui| {
                if ui
                    .selectable_label(*view_mode == ViewMode::Select, "☝")
                    .on_hover_text("Select mode")
                    .clicked()
                {
                    *view_mode = ViewMode::Select;
                }

                if ui
                    .selectable_label(*view_mode == ViewMode::Pan, "✋")
                    .on_hover_text("Pan mode")
                    .clicked()
                {
                    *view_mode = ViewMode::Pan;
                }

                if ui.button("⛶").on_hover_text("Center view").clicked() {
                    let (mut orthographic_projection, mut transform) = camera_query.single_mut();

                    orthographic_projection.scale = 1.0;
                    transform.translation = Vec3::new(0.0, 0.0, transform.translation.z);
                }
            });
        });
}
