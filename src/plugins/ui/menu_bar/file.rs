use crate::plugins::file::{os, FileActions, FileHandleAction, Save};
use crate::utils::event_set::SendEvent;
use bevy::ecs::system::SystemParam;
use bevy_egui::egui;

#[derive(SystemParam)]
pub struct FileSystemParams<'w> {
    file_actions: FileActions<'w>,
    file_handles: os::FileHandlesResource<'w>,
}

pub fn ui(ui: &mut egui::Ui, mut params: FileSystemParams) {
    ui.menu_button("File", |ui| {
        if ui.button("Open...").clicked() {
            ui.close_menu();
            params.file_actions.dispatch(FileHandleAction::Open);
        }

        if ui
            .add_enabled(params.file_handles.ron.is_some(), egui::Button::new("Save"))
            .clicked()
        {
            ui.close_menu();
            params.file_actions.dispatch(Save);
        }

        if ui.button("Save as...").clicked() {
            ui.close_menu();
            params.file_actions.dispatch(FileHandleAction::Save);
        }

        if ui.button("Export as SVG...").clicked() {
            ui.close_menu();
            params.file_actions.dispatch(FileHandleAction::Export);
        }
    });
}
