use crate::selection::Selected;
use crate::style::Styles;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::DrawMode;

pub struct ColorPlugin;

impl Plugin for ColorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::PostUpdate, update_color_from_styles)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                remove_selection_color.after(update_color_from_styles),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                set_selection_color.after(remove_selection_color),
            );
    }
}

#[derive(SystemParam)]
struct DrawModeParams<'w, 's> {
    draw_mode_query: Query<'w, 's, &'static mut DrawMode>,
    children_query: Query<'w, 's, &'static Children>,
}

impl<'w, 's> DrawModeParams<'w, 's> {
    fn set_color_for_entity_and_children(&mut self, entity: Entity, color: Color) {
        let children = self.children_query.iter_descendants(entity);
        let entities = std::iter::once(entity).chain(children);
        let mut iter = self.draw_mode_query.iter_many_mut(entities);

        while let Some(mut draw_mode) = iter.fetch_next() {
            *draw_mode = update_draw_mode_color(*draw_mode, color);
        }
    }
}

fn set_selection_color(
    new_selection_query: Query<Entity, Added<Selected>>,
    mut draw_mode_params: DrawModeParams,
    styles: Res<Styles>,
) {
    if let Ok(new_selection) = new_selection_query.get_single() {
        draw_mode_params.set_color_for_entity_and_children(new_selection, styles.selection_color);
    }
}

fn remove_selection_color(
    deselected: RemovedComponents<Selected>,
    mut draw_mode_params: DrawModeParams,
    styles: Res<Styles>,
) {
    for deselected_entity in deselected.iter() {
        draw_mode_params.set_color_for_entity_and_children(deselected_entity, styles.svg_color);
    }
}

fn update_color_from_styles(
    selection_query: Query<Entity, With<Selected>>,
    mut draw_mode_params: DrawModeParams,
    styles: Res<Styles>,
) {
    if !styles.is_changed() {
        return;
    }

    for mut draw_mode in draw_mode_params.draw_mode_query.iter_mut() {
        *draw_mode = update_draw_mode_color(*draw_mode, styles.svg_color);
    }

    if let Ok(selection) = selection_query.get_single() {
        draw_mode_params.set_color_for_entity_and_children(selection, styles.selection_color);
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
