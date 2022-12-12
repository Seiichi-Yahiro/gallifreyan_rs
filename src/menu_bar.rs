use crate::ui::UiStage;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy_egui::{egui, EguiContext};
use futures_lite::future;
use std::path::PathBuf;

pub struct MenuBarPlugin;

impl Plugin for MenuBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(UiStage, ui.label(UiSystemLabel))
            .add_system(open_file)
            .add_system(save_as)
            .add_system(export);
    }
}

#[derive(SystemLabel)]
pub struct UiSystemLabel;

#[derive(Component)]
struct OpenFile(Task<Option<PathBuf>>);
#[derive(Component)]
struct SaveAs(Task<Option<PathBuf>>);
#[derive(Component)]
struct Export(Task<Option<PathBuf>>);

fn ui(mut commands: Commands, mut egui_context: ResMut<EguiContext>) {
    egui::TopBottomPanel::top("top_bar").show(egui_context.ctx_mut(), |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Open...").clicked() {
                let thread_pool = AsyncComputeTaskPool::get();
                let task = thread_pool.spawn(async move {
                    rfd::FileDialog::new()
                        .add_filter("ron", &["ron"])
                        .pick_file()
                });

                commands.spawn(OpenFile(task));

                ui.close_menu();
            }

            if ui.button("Save").clicked() {
                ui.close_menu();
            }

            if ui.button("Save as...").clicked() {
                let thread_pool = AsyncComputeTaskPool::get();
                let task = thread_pool.spawn(async move {
                    rfd::FileDialog::new()
                        .add_filter("ron", &["ron"])
                        .save_file()
                });

                commands.spawn(SaveAs(task));

                ui.close_menu();
            }

            if ui.button("Export as SVG...").clicked() {
                let thread_pool = AsyncComputeTaskPool::get();
                let task = thread_pool.spawn(async move {
                    rfd::FileDialog::new()
                        .add_filter("svg", &["svg"])
                        .save_file()
                });

                commands.spawn(Export(task));

                ui.close_menu();
            }
        });
    });
}

fn open_file(mut commands: Commands, mut tasks: Query<(Entity, &mut OpenFile)>) {
    for (entity, mut open_file) in tasks.iter_mut() {
        if let Some(result) = future::block_on(future::poll_once(&mut open_file.0)) {
            println!("{:?}", result);
            commands.entity(entity).despawn();
        }
    }
}

fn save_as(mut commands: Commands, mut tasks: Query<(Entity, &mut SaveAs)>) {
    for (entity, mut open_file) in tasks.iter_mut() {
        if let Some(result) = future::block_on(future::poll_once(&mut open_file.0)) {
            println!("{:?}", result);
            commands.entity(entity).despawn();
        }
    }
}

fn export(mut commands: Commands, mut tasks: Query<(Entity, &mut Export)>) {
    for (entity, mut open_file) in tasks.iter_mut() {
        if let Some(result) = future::block_on(future::poll_once(&mut open_file.0)) {
            println!("{:?}", result);
            commands.entity(entity).despawn();
        }
    }
}
