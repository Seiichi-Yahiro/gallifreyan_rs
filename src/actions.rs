use crate::event_set::*;
use crate::image_types::Sentence;
use crate::text_converter;
use crate::ui::UiState;
use bevy::prelude::*;

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event_set::<Actions>().add_system(set_text_action);
    }
}

event_set!(Actions { SetText });

pub struct SetText(pub String);

fn set_text_action(
    mut commands: Commands,
    mut events: EventReader<SetText>,
    mut ui_state: ResMut<UiState>,
    query: Query<Entity, With<Sentence>>,
) {
    if let Some(SetText(text)) = events.iter().last() {
        if let Ok(entity) = query.get_single() {
            commands.entity(entity).despawn_recursive();
        }

        if !text.is_empty() {
            ui_state.tree = Some(text_converter::spawn_sentence(&mut commands, text));
        } else {
            ui_state.tree = None;
        }
    }
}
