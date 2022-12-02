use bevy::app::{App, Plugin};
use bevy::prelude::*;

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Select>()
            .init_resource::<Selection>()
            .add_system(handle_select_events);
    }
}

pub struct Select(pub Option<Entity>);

#[derive(Default, Resource, Deref, DerefMut)]
pub struct Selection(pub Option<Entity>);

fn handle_select_events(mut events: EventReader<Select>, mut selection: ResMut<Selection>) {
    for &Select(new_selection) in events.iter() {
        if **selection != new_selection {
            **selection = new_selection;
        }
    }
}
