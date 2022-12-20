mod file;
mod view;

use crate::menu_bar::view::ViewSystemParams;
use crate::ui::UiStage;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use file::FileSystemParams;

pub struct MenuBarPlugin;

impl Plugin for MenuBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(UiStage, ui.label(UiSystemLabel))
            .add_plugin(file::FilePlugin);
    }
}

#[derive(SystemLabel)]
pub struct UiSystemLabel;

fn ui(
    mut egui_context: ResMut<EguiContext>,
    file_system_params: FileSystemParams,
    view_system_params: ViewSystemParams,
) {
    egui::TopBottomPanel::top("menu_bar").show(egui_context.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            file::ui(ui, file_system_params);
            view::ui(ui, view_system_params);
        });
    });
}
