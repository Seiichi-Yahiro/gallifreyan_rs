use super::camera::WorldCursor;
use super::interaction::Interaction;
use crate::image_types::PositionData;
use crate::math::angle::{Angle, Radian};
use crate::selection::{Select, Selected};
use crate::svg_view::ViewMode;
use bevy::prelude::*;
use bevy_egui::EguiContext;

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(ViewMode::Select)
                .with_system(select_on_click)
                .with_system(drag.before(select_on_click)),
        );
    }
}

fn select_on_click(
    mut events: EventWriter<Select>,
    world_cursor: Res<WorldCursor>,
    egui_context: Res<EguiContext>,
    mouse_button_input: Res<Input<MouseButton>>,
    hit_box_query: Query<(Entity, &Interaction)>,
) {
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }

    let ctx = egui_context.ctx();

    if ctx.is_pointer_over_area() || ctx.is_using_pointer() {
        return;
    }

    let clicked_entity = get_clicked_entity(&hit_box_query, world_cursor.pos);

    if let Some(entity) = clicked_entity {
        events.send(Select(Some(entity)));
    } else {
        events.send(Select(None));
    }
}

fn drag(
    world_cursor: Res<WorldCursor>,
    egui_context: Res<EguiContext>,
    hit_box_query: Query<(Entity, &Interaction)>,
    mut selected_query: Query<
        (Entity, Option<&Parent>, &Transform, &mut PositionData),
        With<Selected>,
    >,
    global_transform_query: Query<&GlobalTransform>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut is_dragging: Local<bool>,
) {
    let (selected_entity, parent, transform, mut position_data) =
        match selected_query.get_single_mut() {
            Ok(it) => it,
            Err(_) => {
                return;
            }
        };

    if mouse_button_input.just_pressed(MouseButton::Left) {
        let ctx = egui_context.ctx();
        let is_ui_blocking =
            ctx.is_pointer_over_area() || ctx.is_using_pointer() || ctx.wants_keyboard_input();

        if is_ui_blocking {
            return;
        }

        let clicked_entity = get_clicked_entity(&hit_box_query, world_cursor.pos);
        *is_dragging = clicked_entity.contains(&selected_entity);
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        *is_dragging = false;
    }

    if !*is_dragging || world_cursor.delta.length_squared() == 0.0 {
        return;
    }

    let parent_rotation = parent
        .and_then(|parent| global_transform_query.get(parent.get()).ok())
        .map(|parent_global_transform| {
            parent_global_transform
                .compute_transform()
                .rotation
                .inverse()
        })
        .unwrap_or(Quat::IDENTITY);

    let rotated_mouse_delta = parent_rotation * world_cursor.delta.extend(0.0);

    let new_transform = Transform::from_translation(rotated_mouse_delta) * *transform;
    let new_position = new_transform.translation.truncate();

    position_data.distance = new_position.length();

    if position_data.distance != 0.0 {
        position_data.angle = Radian::angle_from_vec(new_position)
            .to_degrees()
            .normalize();
    }
}

fn get_clicked_entity(
    hit_box_query: &Query<(Entity, &Interaction)>,
    world_cursor_pos: Vec2,
) -> Option<Entity> {
    hit_box_query
        .iter()
        .filter_map(|(entity, interaction)| {
            if interaction.is_inside(world_cursor_pos) {
                Some((entity, interaction.z))
            } else {
                None
            }
        })
        .max_by(|(_, za), (_, zb)| za.partial_cmp(zb).unwrap())
        .map(|(entity, _)| entity)
}
