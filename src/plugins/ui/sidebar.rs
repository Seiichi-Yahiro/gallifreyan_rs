mod selection;
mod text_input;
mod tree;

use super::UiStage;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use selection::{ui_selection, SelectionSystemParams};
use text_input::{ui_text_input, TextInputSystemParams, TextState};
use tree::{add_is_open_component, ui_tree, TreeSystemParams};

pub struct SideBarPlugin;

impl Plugin for SideBarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TextState>()
            .add_system_to_stage(
                UiStage,
                ui.label(UiSystemLabel)
                    .after(super::menu_bar::UiSystemLabel),
            )
            .add_system(add_is_open_component);
    }
}

#[derive(SystemLabel)]
pub struct UiSystemLabel;

fn ui(
    mut egui_context: ResMut<EguiContext>,
    text_input_system_params: TextInputSystemParams,
    tree_system_params: TreeSystemParams,
    selection_system_params: SelectionSystemParams,
    windows: Res<Windows>,
) {
    let window = windows.primary();
    let side_bar_width = window.width() * 0.2;

    egui::SidePanel::left("sidebar")
        .resizable(true)
        .default_width(side_bar_width)
        .width_range(100.0..=side_bar_width)
        .show(egui_context.ctx_mut(), |ui| {
            ui_text_input(ui, text_input_system_params);
            ui_selection(ui, selection_system_params);
            ui_tree(ui, tree_system_params);
        });
}
