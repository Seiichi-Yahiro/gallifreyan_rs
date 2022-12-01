use bevy::app::{App, Plugin};
use bevy::prelude::Entity;

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Select>();
    }
}

pub struct Select(pub Option<Entity>);
