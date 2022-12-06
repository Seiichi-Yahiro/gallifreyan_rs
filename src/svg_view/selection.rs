use crate::events::{Select, Selection};
use crate::image_types::Radius;
use crate::style::Styles;
use crate::svg_view::camera::SVGViewCamera;
use crate::svg_view::ViewMode;
use bevy::prelude::*;
use bevy_egui::EguiContext;
use bevy_prototype_lyon::prelude::DrawMode;

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(set_select_color)
            .add_system_set(SystemSet::on_update(ViewMode::Select).with_system(select_on_click));
    }
}

fn set_select_color(
    mut draw_mode_query: Query<&mut DrawMode>,
    children_query: Query<&Children, With<DrawMode>>,
    selection: Res<Selection>,
    styles: Res<Styles>,
) {
    if !selection.is_changed() {
        return;
    }

    reset_select_color(&mut draw_mode_query, styles.svg_color);

    if let Some(selected_entity) = **selection {
        set_select_color_recursive(
            selected_entity,
            &mut draw_mode_query,
            &children_query,
            styles.selection_color,
        );
    }
}

fn set_select_color_recursive(
    entity: Entity,
    draw_mode_query: &mut Query<&mut DrawMode>,
    children_query: &Query<&Children, With<DrawMode>>,
    color: Color,
) {
    if let Ok(mut draw_mode) = draw_mode_query.get_mut(entity) {
        *draw_mode = update_draw_mode_color(*draw_mode, color);
    } else {
        return;
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children {
            set_select_color_recursive(*child, draw_mode_query, children_query, color);
        }
    }
}

fn reset_select_color(draw_mode_query: &mut Query<&mut DrawMode>, color: Color) {
    for mut draw_mode in draw_mode_query.iter_mut() {
        *draw_mode = update_draw_mode_color(*draw_mode, color);
    }
}

fn update_draw_mode_color(draw_mode: DrawMode, color: Color) -> DrawMode {
    match draw_mode {
        DrawMode::Fill(mut fill_mode) => {
            fill_mode.color = color;
            DrawMode::Fill(fill_mode)
        }
        DrawMode::Stroke(mut stroke_mode) => {
            stroke_mode.color = color;
            DrawMode::Stroke(stroke_mode)
        }
        DrawMode::Outlined { .. } => draw_mode,
    }
}

fn select_on_click(
    mut events: EventWriter<Select>,
    egui_context: Res<EguiContext>,
    mouse_button_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    camera_query: Query<(&Camera, &GlobalTransform), With<SVGViewCamera>>,
    circle_query: Query<(Entity, &Radius, &GlobalTransform)>,
) {
    let ctx = egui_context.ctx();

    if ctx.is_pointer_over_area() || ctx.is_using_pointer() {
        return;
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        let window = windows.primary();
        let cursor_pos = window.cursor_position();

        let ray: Option<Ray> = cursor_pos.and_then(|cursor_pos| {
            camera_query
                .get_single()
                .ok()
                .and_then(|(camera, global_transform)| {
                    let viewport_pos = camera
                        .logical_viewport_rect()
                        .map(|(min, _max)| min)
                        .unwrap_or(Vec2::ZERO);

                    camera.viewport_to_world(global_transform, cursor_pos - viewport_pos)
                })
        });

        let clicked_entity: Option<Entity> = ray.and_then(|ray| {
            circle_query
                .iter()
                .map(|(entity, radius, global_transform)| {
                    let circle_translation = global_transform.translation();

                    let cursor_pos = (ray.origin + ray.direction * circle_translation.z).truncate();
                    let circle_pos = circle_translation.truncate();

                    let distance: f32 = (circle_pos - cursor_pos).length() - **radius;

                    (entity, distance, circle_translation.z)
                })
                .filter(|(_, distance, _)| *distance <= 0.0)
                .max_by(|(_, _, za), (_, _, zb)| za.partial_cmp(zb).unwrap())
                .map(|(entity, _, _)| entity)
        });

        if let Some(entity) = clicked_entity {
            events.send(Select(Some(entity)));
        } else {
            events.send(Select(None));
        }
    }
}
