mod text_converter;
mod text_input;
mod tree;

use crate::sidebar::text_converter::TextConverterPlugin;
use crate::sidebar::text_input::{ui_text_input, TextInputSystemParams, TextState};
use crate::sidebar::tree::{add_is_open_component, ui_tree, TreeSystemParams};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

pub struct SideBarPlugin;

impl Plugin for SideBarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TextState>()
            .add_system(ui.label(UiSystemLabel))
            .add_system(add_is_open_component)
            .add_plugin(TextConverterPlugin);
    }
}

#[derive(SystemLabel)]
pub struct UiSystemLabel;

fn ui(
    mut egui_context: ResMut<EguiContext>,
    text_input_system_params: TextInputSystemParams,
    tree_system_params: TreeSystemParams,
) {
    egui::SidePanel::left("sidebar")
        .resizable(true)
        .show(egui_context.ctx_mut(), |ui| {
            ui_text_input(ui, text_input_system_params);

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui_tree(ui, tree_system_params);
            });
        });
}
