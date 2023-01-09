use bevy::app::{App, Plugin};
use bevy::ecs::query::QuerySingleError;
use bevy::prelude::*;

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Select>().add_system(handle_select_events);
    }
}

pub struct Select(pub Option<Entity>);

#[derive(Copy, Clone, Component)]
#[component(storage = "SparseSet")]
pub struct Selected;

fn handle_select_events(
    mut commands: Commands,
    mut events: EventReader<Select>,
    selected_query: Query<Entity, With<Selected>>,
) {
    if let Some(&Select(new_selection)) = events.iter().last() {
        match (selected_query.get_single(), new_selection) {
            (Ok(old_selection), Some(new_selection)) => {
                if old_selection != new_selection {
                    debug!(
                        "Update selection: {:?} -> {:?}",
                        old_selection, new_selection
                    );
                    commands.entity(old_selection).remove::<Selected>();
                    commands.entity(new_selection).insert(Selected);
                }
            }
            (Ok(old_selection), None) => {
                debug!("Deselect: {:?}", old_selection);
                commands.entity(old_selection).remove::<Selected>();
            }
            (Err(QuerySingleError::NoEntities(_)), Some(new_selection)) => {
                debug!("Select: {:?}", new_selection);
                commands.entity(new_selection).insert(Selected);
            }
            (Err(QuerySingleError::MultipleEntities(_)), _) => {
                error!("More than one selected entity!");
            }
            _ => {}
        }
    }
}
