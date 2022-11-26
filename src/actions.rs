mod reset;

use crate::actions::reset::*;
use crate::event_set::*;
use crate::image_types::Sentence;
use crate::text_converter;
use bevy::prelude::*;

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event_set::<Actions>()
            .add_system_to_stage(CoreStage::PreUpdate, set_text_action)
            .add_plugin(ResetPlugin);
    }
}

pub struct SetText(pub String);

event_set!(Actions { SetText, ResetAll });

fn set_text_action(
    mut commands: Commands,
    mut events: EventReader<SetText>,
    mut actions: EventWriter<ResetAll>,
    sentence_query: Query<Entity, With<Sentence>>,
) {
    if let Some(SetText(text)) = events.iter().last() {
        for sentence in sentence_query.iter() {
            commands.entity(sentence).despawn_recursive();
        }

        if !text.is_empty() {
            text_converter::spawn_sentence(&mut commands, text);
            actions.send(ResetAll);
        }
    }
}
