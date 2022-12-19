use crate::event_set::SendEvent;
use crate::image_types::{Dot, Letter, LineSlot, Sentence, Word};
use crate::menu_bar::svg_export::{convert_to_svg, SVGQueries};
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use futures::channel::oneshot;
use wasm_bindgen::prelude::*;

pub struct WasmPlugin;

impl Plugin for WasmPlugin {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<FileHandles>()
            .init_non_send_resource::<FileHandleReceiver>()
            .add_system(handle_file_handle_action_event)
            .add_system(receive_file_handle.after(handle_file_handle_action_event))
            .add_system(handle_save_event.after(receive_file_handle))
            .add_system(handle_export_event.after(receive_file_handle));
    }
}

#[wasm_bindgen(module = "/src/menu_bar/web.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn openRONFile() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn saveRONFile() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn saveSVGFile() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn saveToFile(file_handle: JsValue, data: String) -> Result<(), JsValue>;
}

#[derive(Default)]
pub struct FileHandles {
    ron: Option<JsValue>,
    svg: Option<JsValue>,
}

impl FileHandles {
    pub fn has_ron(&self) -> bool {
        self.ron.is_some()
    }
}

type ReceiverType = (JsValue, super::FileHandleAction);

#[derive(Default)]
struct FileHandleReceiver(Option<oneshot::Receiver<ReceiverType>>);

fn handle_file_handle_action_event(
    mut events: EventReader<super::FileHandleAction>,
    mut file_handle_receiver: NonSendMut<FileHandleReceiver>,
) {
    if let Some(&action) = events.iter().last() {
        let (sender, receiver) = oneshot::channel::<ReceiverType>();

        let task = async move {
            let file_handle = match action {
                super::FileHandleAction::Open => openRONFile().await,
                super::FileHandleAction::Save => saveRONFile().await,
                super::FileHandleAction::Export => saveSVGFile().await,
            };

            match file_handle {
                Ok(file_handle) => {
                    if sender.send((file_handle, action)).is_err() {
                        error!("Couldn't send path buffer from open event because receiver was already closed!");
                    }
                }
                Err(error) => {
                    error!("{:?}", error);
                }
            }
        };

        AsyncComputeTaskPool::get().spawn_local(task).detach();
        file_handle_receiver.0 = Some(receiver);
    }
}

fn receive_file_handle(
    mut file_handle_receiver: NonSendMut<FileHandleReceiver>,
    mut file_handles: NonSendMut<FileHandles>,
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
    file_handles: NonSend<FileHandles>,
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
        if let Some(file_handle) = file_handles.ron.clone() {
            let mut builder = DynamicSceneBuilder::from_world(world);
            builder.extract_entities(serialize_query.iter());
            let scene = builder.build();

            let type_registry = world.resource::<AppTypeRegistry>();

            match scene.serialize_ron(type_registry) {
                Ok(data) => {
                    save_to_file(file_handle, data);
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
    file_handles: NonSend<FileHandles>,
    svg_queries: SVGQueries,
) {
    if events.iter().last().is_some() {
        if let Some(path_buffer) = file_handles.svg.clone() {
            let svg = convert_to_svg(svg_queries).build();
            save_to_file(path_buffer, svg);
        }
    }
}

fn save_to_file(file_handle: JsValue, content: String) {
    AsyncComputeTaskPool::get()
        .spawn_local(async move {
            if let Err(error) = saveToFile(file_handle, content).await {
                let msg = format!("{:?}", error);

                error!(msg);

                rfd::AsyncMessageDialog::new()
                    .set_title("Failed to save file")
                    .set_description(&msg)
                    .set_buttons(rfd::MessageButtons::Ok)
                    .set_level(rfd::MessageLevel::Error)
                    .show()
                    .await;
            }
        })
        .detach();
}
