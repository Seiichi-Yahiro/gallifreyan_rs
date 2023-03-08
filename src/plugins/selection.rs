use crate::math::angle::{Angle, Radian};
use crate::plugins::color_theme::{ColorDependency, ColorTheme, DRAW_COLOR, SELECT_COLOR};
use crate::plugins::interaction::Interaction;
use crate::plugins::svg_view::{ViewMode, WorldCursor};
use crate::plugins::text_converter::components::PositionData;
use bevy::app::{App, Plugin};
use bevy::ecs::query::QuerySingleError;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_prototype_lyon::prelude::{Fill, Stroke};

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Select>()
            .add_system(handle_select_events)
            .add_systems(
                (remove_selection_color, set_selection_color)
                    .chain()
                    .in_base_set(CoreSet::PostUpdate),
            )
            .add_systems(
                (drag, select_on_click)
                    .chain()
                    .in_set(OnUpdate(ViewMode::Select)),
            );
    }
}

pub struct Select(pub Option<Entity>);

#[derive(Copy, Clone, Component)]
#[component(storage = "SparseSet")]
pub struct Selected;

fn handle_select_events(
    mut commands: Commands,
    mut events: EventReader<Select>,
    selected_query: Query<Entity, With<Selected>>,
) {
    if let Some(&Select(new_selection)) = events.iter().last() {
        match (selected_query.get_single(), new_selection) {
            (Ok(old_selection), Some(new_selection)) => {
                if old_selection != new_selection {
                    debug!(
                        "Update selection: {:?} -> {:?}",
                        old_selection, new_selection
                    );
                    commands.entity(old_selection).remove::<Selected>();
                    commands.entity(new_selection).insert(Selected);
                }
            }
            (Ok(old_selection), None) => {
                debug!("Deselect: {:?}", old_selection);
                commands.entity(old_selection).remove::<Selected>();
            }
            (Err(QuerySingleError::NoEntities(_)), Some(new_selection)) => {
                debug!("Select: {:?}", new_selection);
                commands.entity(new_selection).insert(Selected);
            }
            (Err(QuerySingleError::MultipleEntities(_)), _) => {
                error!("More than one selected entity!");
            }
            _ => {}
        }
    }
}

#[derive(SystemParam)]
struct DrawModeParams<'w, 's> {
    draw_mode_query: Query<
        'w,
        's,
        (
            &'static mut ColorDependency,
            Option<&'static mut Stroke>,
            Option<&'static mut Fill>,
        ),
    >,
    children_query: Query<'w, 's, &'static Children>,
    color_theme: Res<'w, ColorTheme>,
}

impl<'w, 's> DrawModeParams<'w, 's> {
    fn set_color_for_entity_and_children(&mut self, entity: Entity, dependency: &'static str) {
        if let Some(new_color) = self.color_theme.get(dependency) {
            let children = self.children_query.iter_descendants(entity);
            let entities = std::iter::once(entity).chain(children);
            let mut iter = self.draw_mode_query.iter_many_mut(entities);

            while let Some((mut color_dependency, mut stroke, mut fill)) = iter.fetch_next() {
                *color_dependency = ColorDependency(dependency);

                if let Some(stroke) = stroke.as_mut() {
                    stroke.color = new_color;
                }

                if let Some(fill) = fill.as_mut() {
                    fill.color = new_color;
                }
            }
        } else {
            error!("Couldn't find {} key in color theme!", dependency);
        }
    }
}

fn set_selection_color(
    new_selection_query: Query<Entity, Added<Selected>>,
    mut draw_mode_params: DrawModeParams,
) {
    if let Ok(new_selection) = new_selection_query.get_single() {
        draw_mode_params.set_color_for_entity_and_children(new_selection, SELECT_COLOR);
    }
}

fn remove_selection_color(
    mut deselected: RemovedComponents<Selected>,
    mut draw_mode_params: DrawModeParams,
) {
    for deselected_entity in &mut deselected {
        draw_mode_params.set_color_for_entity_and_children(deselected_entity, DRAW_COLOR);
    }
}

fn select_on_click(
    mut events: EventWriter<Select>,
    world_cursor: Res<WorldCursor>,
    egui_contexts: EguiContexts,
    mouse_button_input: Res<Input<MouseButton>>,
    hit_box_query: Query<(Entity, &Interaction)>,
) {
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }

    let ctx = egui_contexts.ctx();

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
    egui_contexts: EguiContexts,
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
        let ctx = egui_contexts.ctx();
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
