use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use futures::channel::oneshot;
use wasm_bindgen::prelude::*;

pub type FileHandle = JsValue;
pub type FileHandlesResource<'w> = NonSend<'w, super::FileHandles>;
pub type FileHandlesResourceMut<'w> = NonSendMut<'w, super::FileHandles>;

pub type FileHandleReceiverResourceMut<'w> = NonSendMut<'w, super::FileHandleReceiver>;

#[wasm_bindgen(module = "/src/menu_bar/file/web.js")]
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

pub fn spawn_file_handle_task(
    action: super::FileHandleAction,
    sender: oneshot::Sender<super::FileHandleChannelType>,
) {
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
}

pub fn save_to_file(file_handle: FileHandle, content: String) {
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
