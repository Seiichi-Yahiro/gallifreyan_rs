mod file;
mod settings;

use crate::menu_bar::settings::SettingsSystemParams;
use crate::ui::UiStage;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use file::FileSystemParams;

pub struct MenuBarPlugin;

impl Plugin for MenuBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(UiStage, ui.label(UiSystemLabel))
            .add_plugin(file::FilePlugin)
            .add_plugin(settings::SettingsPlugin);
    }
}

#[derive(SystemLabel)]
pub struct UiSystemLabel;

fn ui(
    mut egui_context: ResMut<EguiContext>,
    file_system_params: FileSystemParams,
    settings_system_params: SettingsSystemParams,
) {
    egui::TopBottomPanel::top("menu_bar").show(egui_context.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            file::ui(ui, file_system_params);
            settings::ui(ui, settings_system_params);
        });
    });
}
