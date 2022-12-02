mod selection;
mod text_converter;
mod text_input;
mod tree;

use crate::sidebar::selection::{ui_selection, SelectionSystemParams};
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
            egui::TopBottomPanel::top("text_input")
                .frame(egui::Frame::none())
                .show_inside(ui, |ui| {
                    ui_text_input(ui, text_input_system_params);
                });

            ui_selection(ui, selection_system_params);

            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui_tree(ui, tree_system_params);
                    });
            });
        });
}
