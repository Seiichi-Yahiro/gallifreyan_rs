mod file;
mod settings;

use super::{UiBaseSet, UiSet};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use file::FileSystemParams;
use settings::SettingsSystemParams;

pub struct MenuBarPlugin;

impl Plugin for MenuBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(ui.in_base_set(UiBaseSet).in_set(UiSet::MenuBar))
            .add_plugin(settings::SettingsPlugin);
    }
}

fn ui(
    mut egui_contexts: EguiContexts,
    file_system_params: FileSystemParams,
    settings_system_params: SettingsSystemParams,
) {
    egui::TopBottomPanel::top("menu_bar").show(egui_contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            file::ui(ui, file_system_params);
            settings::ui(ui, settings_system_params);
        });
    });
}
