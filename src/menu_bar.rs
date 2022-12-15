use crate::ui::UiStage;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bevy_egui::{egui, EguiContext};
use futures::channel::oneshot::{channel, Receiver};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

pub struct MenuBarPlugin;

impl Plugin for MenuBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(UiStage, ui.label(UiSystemLabel))
            .add_system(load_file);
    }
}

#[derive(SystemLabel)]
pub struct UiSystemLabel;

#[derive(Component, Deref, DerefMut)]
struct LoadFile(Receiver<String>);

fn ui(mut commands: Commands, mut egui_context: ResMut<EguiContext>) {
    egui::TopBottomPanel::top("top_bar").show(egui_context.ctx_mut(), |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Open...").clicked() {
                ui.close_menu();
                open_file(&mut commands);
            }

            if ui.button("Save").clicked() {
                ui.close_menu();
            }

            if ui.button("Save as...").clicked() {
                ui.close_menu();
            }

            if ui.button("Export as SVG...").clicked() {
                ui.close_menu();
            }
        });
    });
}

fn open_file(commands: &mut Commands) {
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
}

#[wasm_bindgen(module = "/wasm/web.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn openFile() -> Result<JsValue, JsValue>;
}
