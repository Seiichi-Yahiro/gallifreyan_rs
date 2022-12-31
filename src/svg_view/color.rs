use crate::events::Selection;
use crate::style::Styles;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::DrawMode;

pub struct ColorPlugin;

impl Plugin for ColorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(set_draw_mode_color);
    }
}

fn set_draw_mode_color(
    mut draw_mode_query: Query<&mut DrawMode>,
    children: Query<&Children>,
    selection: Res<Selection>,
    styles: Res<Styles>,
) {
    if !selection.is_changed() && !styles.is_changed() {
        return;
    }

    reset_select_color(&mut draw_mode_query, styles.svg_color);

    if let Some(selected_entity) = **selection {
        set_select_color(
            selected_entity,
            &mut draw_mode_query,
            &children,
            styles.selection_color,
        );
    }
}

fn set_select_color(
    entity: Entity,
    draw_mode_query: &mut Query<&mut DrawMode>,
    children: &Query<&Children>,
    color: Color,
) {
    if let Ok(mut draw_mode) = draw_mode_query.get_mut(entity) {
        *draw_mode = update_draw_mode_color(*draw_mode, color);

        for child in children.iter_descendants(entity) {
            if let Ok(mut draw_mode) = draw_mode_query.get_mut(child) {
                *draw_mode = update_draw_mode_color(*draw_mode, color);
            }
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
