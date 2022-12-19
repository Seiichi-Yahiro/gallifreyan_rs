use crate::event_set::SendEvent;
use crate::image_types::{Dot, Letter, LineSlot, Sentence, Word};
use crate::menu_bar::svg_export::{convert_to_svg, SVGQueries};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, IoTaskPool};
use futures::channel::oneshot;
use std::path::PathBuf;

pub struct NativePlugin;

impl Plugin for NativePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FileHandles>()
            .init_resource::<FileHandleReceiver>()
            .add_system(handle_file_handle_action_event)
            .add_system(receive_file_handle.after(handle_file_handle_action_event))
            .add_system(handle_save_event.after(receive_file_handle))
            .add_system(handle_export_event.after(receive_file_handle));
    }
}

#[derive(Default, Resource)]
pub struct FileHandles {
    ron: Option<PathBuf>,
    svg: Option<PathBuf>,
}

impl FileHandles {
    pub fn has_ron(&self) -> bool {
        self.ron.is_some()
    }
}

type FileHandleReceiverType = (PathBuf, super::FileHandleAction);

#[derive(Default, Resource)]
struct FileHandleReceiver(Option<oneshot::Receiver<FileHandleReceiverType>>);

const RON: &str = "Rusty Object Notation";
const RON_EXTENSIONS: &[&str] = &["ron", "txt"];

const SVG: &str = "Scalable Vector Graphics";
const SVG_EXTENSIONS: &[&str] = &["svg"];

fn handle_file_handle_action_event(
    mut events: EventReader<super::FileHandleAction>,
    mut file_handle_receiver: ResMut<FileHandleReceiver>,
) {
    if let Some(&action) = events.iter().last() {
        let (sender, receiver) = oneshot::channel::<FileHandleReceiverType>();

        let task = async move {
            let file_dialog = rfd::FileDialog::new();

            let path_buffer = match action {
                super::FileHandleAction::Open => {
                    file_dialog.add_filter(RON, RON_EXTENSIONS).pick_file()
                }
                super::FileHandleAction::Save => {
                    file_dialog.add_filter(RON, RON_EXTENSIONS).save_file()
                }
                super::FileHandleAction::Export => {
                    file_dialog.add_filter(SVG, SVG_EXTENSIONS).save_file()
                }
            };

            if let Some(path_buffer) = path_buffer {
                if sender.send((path_buffer, action)).is_err() {
                    error!("Couldn't send path buffer from open event because receiver was already closed!");
                }
            }
        };

        AsyncComputeTaskPool::get().spawn(task).detach();
        file_handle_receiver.0 = Some(receiver);
    }
}

fn receive_file_handle(
    mut file_handle_receiver: ResMut<FileHandleReceiver>,
    mut file_handles: ResMut<FileHandles>,
    mut file_actions: super::FileActions,
) {
    if let Some(mut receiver) = file_handle_receiver.0.take() {
        match receiver.try_recv() {
            Ok(Some((path_buffer, action))) => match action {
                super::FileHandleAction::Open => {
                    file_handles.ron = Some(path_buffer);
                    file_actions.dispatch(super::Load);
                }
                super::FileHandleAction::Save => {
                    file_handles.ron = Some(path_buffer);
                    file_actions.dispatch(super::Save);
                }
                super::FileHandleAction::Export => {
                    file_handles.svg = Some(path_buffer);
                    file_actions.dispatch(super::Export);
                }
            },
            Ok(None) => {
                file_handle_receiver.0 = Some(receiver);
            }
            Err(_canceled) => {}
        }
    }
}

fn handle_save_event(
    world: &World,
    mut events: EventReader<super::Save>,
    file_handles: Res<FileHandles>,
    serialize_query: Query<
        Entity,
        Or<(
            With<Sentence>,
            With<Word>,
            With<Letter>,
            With<Dot>,
            With<LineSlot>,
        )>,
    >,
) {
    if events.iter().last().is_some() {
        if let Some(path_buffer) = file_handles.ron.clone() {
            let mut builder = DynamicSceneBuilder::from_world(world);
            builder.extract_entities(serialize_query.iter());
            let scene = builder.build();

            let type_registry = world.resource::<AppTypeRegistry>();

            match scene.serialize_ron(type_registry) {
                Ok(data) => {
                    save_to_file(path_buffer, data);
                }
                Err(error) => {
                    error!("{}", error);
                }
            }
        }
    }
}

fn handle_export_event(
    mut events: EventReader<super::Export>,
    file_handles: Res<FileHandles>,
    svg_queries: SVGQueries,
) {
    if events.iter().last().is_some() {
        if let Some(path_buffer) = file_handles.svg.clone() {
            let svg = convert_to_svg(svg_queries).build();
            save_to_file(path_buffer, svg);
        }
    }
}

fn save_to_file(path_buffer: PathBuf, content: String) {
    IoTaskPool::get()
        .spawn(async move {
            if let Err(error) = std::fs::write(path_buffer, content) {
                let msg = format!("{}", error);

                error!(msg);

                rfd::MessageDialog::new()
                    .set_title("Failed to save file")
                    .set_description(&msg)
                    .set_buttons(rfd::MessageButtons::Ok)
                    .set_level(rfd::MessageLevel::Error)
                    .show();
            }
        })
        .detach();
}
