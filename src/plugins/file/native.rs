use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, IoTaskPool};
use futures::channel::oneshot;

pub type FileHandle = std::path::PathBuf;
pub type FileHandlesResource<'w> = Res<'w, super::FileHandles>;
pub type FileHandlesResourceMut<'w> = ResMut<'w, super::FileHandles>;

pub type FileHandleReceiverResourceMut<'w> = ResMut<'w, super::FileHandleReceiver>;

pub fn spawn_file_handle_task(
    action: super::FileHandleAction,
    sender: oneshot::Sender<super::FileHandleChannelType>,
) {
    const RON: &str = "Rusty Object Notation";
    const RON_EXTENSIONS: &[&str] = &["ron", "txt"];

    const SVG: &str = "Scalable Vector Graphics";
    const SVG_EXTENSIONS: &[&str] = &["svg"];

    let task = async move {
        let file_dialog = rfd::FileDialog::new();

        let file_handle = match action {
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

        if let Some(file_handle) = file_handle {
            if sender.send((file_handle, action)).is_err() {
                error!(
                "Couldn't send path buffer from open event because receiver was already closed!"
            );
            }
        }
    };

    AsyncComputeTaskPool::get().spawn(task).detach();
}

pub fn save_to_file(file_handle: FileHandle, content: String) {
    IoTaskPool::get()
        .spawn(async move {
            if let Err(error) = std::fs::write(file_handle.clone(), content) {
                let msg = format!("{}", error);

                error!(msg);

                rfd::MessageDialog::new()
                    .set_title("Failed to save file")
                    .set_description(&msg)
                    .set_buttons(rfd::MessageButtons::Ok)
                    .set_level(rfd::MessageLevel::Error)
                    .show();
            } else {
                info!("Successfully wrote to file: {:?}", file_handle);
            }
        })
        .detach();
}
