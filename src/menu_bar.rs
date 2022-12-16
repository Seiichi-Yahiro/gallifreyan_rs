#[cfg(not(target_arch = "wasm32"))]
mod native;

//#[cfg(target_arch = "wasm32")]
mod wasm;

use crate::event_set::*;
use crate::ui::UiStage;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

pub struct MenuBarPlugin;

impl Plugin for MenuBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_event_set::<FileActions>()
            .add_system_to_stage(UiStage, ui.label(UiSystemLabel));

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

fn ui(mut egui_context: ResMut<EguiContext>, mut file_actions: FileActions) {
    egui::TopBottomPanel::top("top_bar").show(egui_context.ctx_mut(), |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Open...").clicked() {
                ui.close_menu();
                file_actions.dispatch(FileHandleAction::Open);
            }

            // TODO disable when no file handle
            if ui.button("Save").clicked() {
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

/*fn open_file(commands: &mut Commands) {
    let (sender, receiver) = channel::<String>();

    #[cfg(not(target_arch = "wasm32"))]
    let task = async move {
        let file = rfd::AsyncFileDialog::new()
            .add_filter("ron", &["ron"])
            .pick_file()
            .await;

        if let Some(file) = file {
            let data = file.read().await;

            if let Ok(file_content) = String::from_utf8(data) {
                sender.send(file_content).ok();
            }
        }
    };

    #[cfg(target_arch = "wasm32")]
    let task = async move {
        if let Ok(array_buffer) = openFile().await {
            let buffer = js_sys::Uint8Array::new(&array_buffer);
            let data: Vec<u8> = buffer.to_vec();

            if let Ok(file_content) = String::from_utf8(data) {
                sender.send(file_content).ok();
            }
        }
    };

    let thread_pool = AsyncComputeTaskPool::get();
    thread_pool.spawn(task).detach();

    commands.spawn(LoadFile(receiver));
}

fn load_file(mut commands: Commands, mut query: Query<(Entity, &mut LoadFile)>) {
    for (entity, mut open_file) in query.iter_mut() {
        match open_file.try_recv() {
            Ok(Some(file_content)) => {
                info!("{}", file_content);
                commands.entity(entity).despawn();
            }
            Ok(None) => { /*not yet received*/ }
            Err(_canceled) => {
                commands.entity(entity).despawn();
            }
        }
    }
}*/
