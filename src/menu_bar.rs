#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(target_arch = "wasm32")]
mod wasm;

use crate::event_set::*;
use crate::ui::UiStage;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

pub struct MenuBarPlugin;

impl Plugin for MenuBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_event_set::<FileActions>()
            .add_system_to_stage(UiStage, can_save.pipe(ui).label(UiSystemLabel));

        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugin(native::NativePlugin);

        #[cfg(target_arch = "wasm32")]
        app.add_plugin(wasm::WasmPlugin);
    }
}

#[derive(SystemLabel)]
pub struct UiSystemLabel;

event_set!(FileActions {
    FileHandleAction,
    Load,
    Save,
    Export
});

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum FileHandleAction {
    Open,
    Save,
    Export,
}

#[derive(Debug, Copy, Clone)]
struct Load;

#[derive(Debug, Copy, Clone)]
struct Save;

#[derive(Debug, Copy, Clone)]
struct Export;

fn can_save(world: &World) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    let file_handle = world.resource::<native::FileHandles>();

    #[cfg(target_arch = "wasm32")]
    let file_handle = world.non_send_resource::<wasm::FileHandles>();

    file_handle.has_ron()
}

fn ui(
    In(can_save): In<bool>,
    mut egui_context: ResMut<EguiContext>,
    mut file_actions: FileActions,
) {
    egui::TopBottomPanel::top("top_bar").show(egui_context.ctx_mut(), |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Open...").clicked() {
                ui.close_menu();
                file_actions.dispatch(FileHandleAction::Open);
            }

            if ui
                .add_enabled(can_save, egui::Button::new("Save"))
                .clicked()
            {
                ui.close_menu();
                file_actions.dispatch(Save);
            }

            if ui.button("Save as...").clicked() {
                ui.close_menu();
                file_actions.dispatch(FileHandleAction::Save);
            }

            if ui.button("Export as SVG...").clicked() {
                ui.close_menu();
                file_actions.dispatch(FileHandleAction::Export);
            }
        });
    });
}
