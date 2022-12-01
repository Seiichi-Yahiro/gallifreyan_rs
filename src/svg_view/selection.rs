use crate::events::Select;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::DrawMode;

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(set_select_color);
    }
}

fn set_select_color(
    mut select_events: EventReader<Select>,
    mut draw_mode_query: Query<&mut DrawMode>,
    children_query: Query<&Children, With<DrawMode>>,
    mut previous_selection: Local<Option<Entity>>,
) {
    for &Select(selection) in select_events.iter() {
        if selection == *previous_selection {
            continue;
        } else {
            *previous_selection = selection;
        }

        reset_select_color(&mut draw_mode_query);

        if let Some(selected_entity) = selection {
            set_select_color_recursive(selected_entity, &mut draw_mode_query, &children_query);
        }
    }
}

fn set_select_color_recursive(
    entity: Entity,
    draw_mode_query: &mut Query<&mut DrawMode>,
    children_query: &Query<&Children, With<DrawMode>>,
) {
    if let Ok(mut draw_mode) = draw_mode_query.get_mut(entity) {
        *draw_mode = match *draw_mode {
            DrawMode::Fill(mut fill_mode) => {
                fill_mode.color = Color::rgb_u8(144, 202, 249);
                DrawMode::Fill(fill_mode)
            }
            DrawMode::Stroke(mut stroke_mode) => {
                stroke_mode.color = Color::rgb_u8(144, 202, 249);
                DrawMode::Stroke(stroke_mode)
            }
            DrawMode::Outlined { .. } => *draw_mode,
        }
    } else {
        return;
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children {
            set_select_color_recursive(*child, draw_mode_query, children_query);
        }
    }
}

fn reset_select_color(draw_mode_query: &mut Query<&mut DrawMode>) {
    for mut draw_mode in draw_mode_query.iter_mut() {
        *draw_mode = match *draw_mode {
            DrawMode::Fill(mut fill_mode) => {
                fill_mode.color = Color::BLACK;
                DrawMode::Fill(fill_mode)
            }
            DrawMode::Stroke(mut stroke_mode) => {
                stroke_mode.color = Color::BLACK;
                DrawMode::Stroke(stroke_mode)
            }
            DrawMode::Outlined { .. } => *draw_mode,
        }
    }
}
