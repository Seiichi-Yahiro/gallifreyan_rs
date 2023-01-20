#[cfg_attr(not(target_arch = "wasm32"), path = "file/native.rs")]
#[cfg_attr(target_arch = "wasm32", path = "file/wasm.rs")]
pub mod os;
mod svg_export;

use crate::plugins::text_converter::components::{
    Dot, Letter, LineSlot, NestedVocalPositionCorrection, Sentence, Word,
};
use crate::utils::event_set::*;
use bevy::prelude::*;
use futures::channel::oneshot;
pub use svg_export::{convert_to_svg, SVGQueries};

pub struct FilePlugin;

impl Plugin for FilePlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_arch = "wasm32"))]
        app.init_resource::<FileHandles>()
            .init_resource::<FileHandleReceiver>();

        #[cfg(target_arch = "wasm32")]
        app.init_non_send_resource::<FileHandles>()
            .init_non_send_resource::<FileHandleReceiver>();

        app.add_event_set::<FileActions>()
            .add_system(handle_file_handle_action_event)
            .add_system(receive_file_handle.after(handle_file_handle_action_event))
            .add_system(handle_save_event.after(receive_file_handle))
            .add_system(handle_export_event.after(receive_file_handle));
    }
}

event_set!(pub FileActions {
    FileHandleAction,
    Load,
    Save,
    Export
});

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FileHandleAction {
    Open,
    Save,
    Export,
}

#[derive(Debug, Copy, Clone)]
pub struct Load;

#[derive(Debug, Copy, Clone)]
pub struct Save;

#[derive(Debug, Copy, Clone)]
pub struct Export;

#[derive(Default)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Resource))]
pub struct FileHandles {
    pub ron: Option<os::FileHandle>,
    pub svg: Option<os::FileHandle>,
}

type FileHandleChannelType = (os::FileHandle, FileHandleAction);

#[derive(Default)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Resource))]
pub struct FileHandleReceiver(Option<oneshot::Receiver<FileHandleChannelType>>);

fn handle_file_handle_action_event(
    mut events: EventReader<FileHandleAction>,
    mut file_handle_receiver: os::FileHandleReceiverResourceMut,
) {
    if let Some(&action) = events.iter().last() {
        let (sender, receiver) = oneshot::channel::<FileHandleChannelType>();
        os::spawn_file_handle_task(action, sender);
        file_handle_receiver.0 = Some(receiver);
    }
}

fn receive_file_handle(
    mut file_handle_receiver: os::FileHandleReceiverResourceMut,
    mut file_handles: os::FileHandlesResourceMut,
    mut file_actions: FileActions,
) {
    if let Some(mut receiver) = file_handle_receiver.0.take() {
        match receiver.try_recv() {
            Ok(Some((path_buffer, action))) => match action {
                FileHandleAction::Open => {
                    file_handles.ron = Some(path_buffer);
                    file_actions.dispatch(Load);
                }
                FileHandleAction::Save => {
                    file_handles.ron = Some(path_buffer);
                    file_actions.dispatch(Save);
                }
                FileHandleAction::Export => {
                    file_handles.svg = Some(path_buffer);
                    file_actions.dispatch(Export);
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
    mut events: EventReader<Save>,
    file_handles: os::FileHandlesResource,
    serialize_query: Query<
        Entity,
        Or<(
            With<Sentence>,
            With<Word>,
            With<Letter>,
            With<NestedVocalPositionCorrection>,
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
                    info!("Save to file: {:?}", path_buffer);
                    os::save_to_file(path_buffer, data);
                }
                Err(error) => {
                    error!("{}", error);
                }
            }
        }
    }
}

fn handle_export_event(
    mut events: EventReader<Export>,
    file_handles: os::FileHandlesResource,
    svg_queries: SVGQueries,
) {
    if events.iter().last().is_some() {
        if let Some(path_buffer) = file_handles.svg.clone() {
            info!("Export to file: {:?}", path_buffer);

            match convert_to_svg(svg_queries) {
                Ok(svg) => {
                    os::save_to_file(path_buffer, svg.to_string());
                }
                Err(err) => {
                    error!("Failed to export file to svg: {}", err);
                }
            }
        }
    }
}
