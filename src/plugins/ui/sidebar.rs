mod selection;
mod text_input;
mod tree;

use super::{UiBaseSet, UiSet};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use selection::{ui_selection, SelectionSystemParams};
use text_input::{ui_text_input, TextInputSystemParams, TextState};
use tree::{add_is_open_component, ui_tree, TreeSystemParams};

pub struct SideBarPlugin;

impl Plugin for SideBarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TextState>()
            .add_system(ui.in_base_set(UiBaseSet).in_set(UiSet::SideBar))
            .add_system(add_is_open_component);
    }
}

fn ui(
    mut egui_contexts: EguiContexts,
    text_input_system_params: TextInputSystemParams,
    tree_system_params: TreeSystemParams,
    selection_system_params: SelectionSystemParams,
    windows: Query<&Window>,
) {
    let window = windows.get_single().expect("Only one Window should exist!");
    let side_bar_width = window.width() * 0.2;

    egui::SidePanel::left("sidebar")
        .resizable(true)
        .default_width(side_bar_width)
        .width_range(100.0..=side_bar_width)
        .show(egui_contexts.ctx_mut(), |ui| {
            ui_text_input(ui, text_input_system_params);
            ui_selection(ui, selection_system_params);
            ui_tree(ui, tree_system_params);
        });
}
