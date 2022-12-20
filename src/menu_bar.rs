mod file;

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

fn ui(mut egui_context: ResMut<EguiContext>, file_system_params: FileSystemParams) {
    egui::TopBottomPanel::top("top_bar").show(egui_context.ctx_mut(), |ui| {
        file::ui(ui, file_system_params);
    });
}
