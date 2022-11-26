use crate::actions::{Actions, SetText, UiSizeChanged};
use crate::event_set::SendEvent;
use crate::text_converter;
use crate::ui::tree::render_tree;
use crate::ui::UiState;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

#[derive(Default, Copy, Clone)]
pub struct UiSize {
    pub sidebar_width: f32,
}

pub fn ui(
    mut egui_context: ResMut<EguiContext>,
    mut ui_state: ResMut<UiState>,
    mut actions: Actions,
    mut local_ui_size: Local<UiSize>,
) {
    let side_bar = egui::SidePanel::left("sidebar")
        .resizable(true)
        .show(egui_context.ctx_mut(), |ui| {
            if ui.text_edit_singleline(&mut ui_state.text).changed() {
                let new_sanitized_text = text_converter::sanitize_text_input(&ui_state.text);
                if ui_state.sanitized_text != new_sanitized_text {
                    ui_state.sanitized_text = new_sanitized_text;
                    actions.dispatch(SetText(ui_state.sanitized_text.clone()));
                }
            }

            if let Some(node) = &ui_state.tree {
                render_tree(node, ui);
            }
        });

    let side_bar_width = side_bar.response.rect.width();

    if side_bar_width != local_ui_size.sidebar_width {
        local_ui_size.sidebar_width = side_bar_width;
        actions.dispatch(UiSizeChanged(*local_ui_size));
    }
}
