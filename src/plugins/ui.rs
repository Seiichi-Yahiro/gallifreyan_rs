mod interactions;
mod text_input;

use crate::plugins::ui::interactions::InteractionsPlugin;
use crate::plugins::ui::text_input::{TextInputPlugin, TextInputWidget};
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
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
