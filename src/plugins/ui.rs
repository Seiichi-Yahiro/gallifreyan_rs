mod interactions;
mod text_input;

use crate::plugins::ui::interactions::InteractionsPlugin;
use crate::plugins::ui::text_input::{TextInputPlugin, TextInputWidget};
use bevy::asset::load_internal_binary_asset;
use bevy::prelude::*;

const FONT_HANDLE: Handle<Font> = Handle::weak_from_u128(10482756907980398621);
const FONT_SIZE: f32 = 16.0;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        load_internal_binary_asset!(
            app,
            FONT_HANDLE,
            "../../assets/Roboto-Regular.ttf",
            |bytes: &[u8], _path: String| { Font::try_from_bytes(bytes.to_vec()).unwrap() }
        );

        app.add_plugins(InteractionsPlugin)
            .add_plugins(TextInputPlugin)
            .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.add(TextInputWidget {
        text: "".to_string(),
    });
}
