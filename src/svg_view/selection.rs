use super::camera::WorldCursor;
use super::interaction::Interaction;
use crate::events::{Select, Selection};
use crate::image_types::PositionData;
use crate::math::angle_from_position;
use crate::style::Styles;
use crate::svg_view::ViewMode;
use bevy::prelude::*;
use bevy_egui::EguiContext;
use bevy_prototype_lyon::prelude::DrawMode;

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(set_select_color).add_system_set(
            SystemSet::on_update(ViewMode::Select)
                .with_system(select_on_click)
                .with_system(drag.before(select_on_click)),
        );
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

    let clicked_entity: Option<Entity> = hit_box_query
        .iter()
        .filter_map(|(entity, interaction)| {
            if interaction.hit_box.is_inside(world_cursor.pos) {
                Some((entity, interaction.z))
            } else {
                None
            }
        })
        .max_by(|(_, za), (_, zb)| za.partial_cmp(zb).unwrap())
        .map(|(entity, _)| entity);

    if let Some(entity) = clicked_entity {
        events.send(Select(Some(entity)));
    } else {
        events.send(Select(None));
    }
}

fn drag(
    world_cursor: Res<WorldCursor>,
    egui_context: Res<EguiContext>,
    mut selected_query: Query<(
        &Interaction,
        &Transform,
        &GlobalTransform,
        &mut PositionData,
    )>,
    selection: Res<Selection>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut is_dragging: Local<bool>,
) {
    if selection.is_none() {
        return;
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        let ctx = egui_context.ctx();
        let is_ui_blocking =
            ctx.is_pointer_over_area() || ctx.is_using_pointer() || ctx.wants_keyboard_input();

        if is_ui_blocking {
            return;
        }

        if let Ok(interaction) = selected_query.get_component::<Interaction>(selection.unwrap()) {
            *is_dragging = interaction.hit_box.is_inside(world_cursor.pos);
        }
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        *is_dragging = false;
    }

    if !*is_dragging || world_cursor.delta.length_squared() == 0.0 {
        return;
    }

    if let Ok((_interaction, transform, global_transform, mut position_data)) =
        selected_query.get_mut(selection.unwrap())
    {
        let global_translation = global_transform.translation().truncate();

        let new_position = global_translation + world_cursor.delta;
        let parent_position = global_translation - transform.translation.truncate();

        let distance_vec = new_position - parent_position;

        position_data.distance = distance_vec.length();

        if position_data.distance != 0.0 {
            position_data.angle = angle_from_position(distance_vec);
        }
    }
}
