use bevy::ecs::query::QuerySingleError;
use bevy::prelude::*;

use super::components::{Text, *};

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct Sentence;

#[derive(Bundle)]
pub struct SentenceBundle {
    pub name: Name,
    pub sentence: Sentence,
    pub text: Text,
    pub words: CircleChildren,
    pub line_slots: LineSlotChildren,
}

impl SentenceBundle {
    pub fn new(sentence: String) -> Self {
        Self {
            name: Name::new("Sentence"),
            sentence: Sentence,
            text: Text(sentence),
            words: CircleChildren::default(),
            line_slots: LineSlotChildren::default(),
        }
    }
}

pub fn convert_sentence(
    mut commands: Commands,
    text: Res<super::Text>,
    mut sentence_query: Query<(Entity, &mut Text), With<Sentence>>,
) {
    match sentence_query.get_single_mut() {
        Ok((sentence_entity, mut sentence_text)) => {
            if text.is_empty() {
                debug!("Despawn sentence: {}", sentence_text.as_str());
                commands.entity(sentence_entity).despawn_recursive();
            } else if **sentence_text != **text {
                debug!(
                    "Update sentence: {} -> {}",
                    sentence_text.as_str(),
                    text.as_str()
                );

                **sentence_text = text.clone();
            }
        }
        Err(QuerySingleError::NoEntities(_)) => {
            debug!("Spawn sentence: {}", text.as_str());
            let bundle = SentenceBundle::new(text.clone());
            commands.spawn(bundle);
        }
        Err(QuerySingleError::MultipleEntities(_)) => {
            error!("Only one sentence is currently supported");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::plugins::text_converter::{
        test::test_component_update, SetText, TextConverterPlugin,
    };

    #[test]
    fn should_spawn_sentence() {
        test_component_update::<Text, Sentence>("my sentence", "my sentence", |before, _after| {
            assert_eq!(before.len(), 1);
            assert_eq!(*before[0], "my sentence");
        });
    }

    #[test]
    fn should_remove_sentence() {
        test_component_update::<Text, Sentence>("my sentence", "", |_before, after| {
            assert_eq!(after.len(), 0);
        });
    }

    #[test]
    fn should_despawn_children() {
        let mut app = App::new();
        app.add_plugins(TextConverterPlugin);

        app.world_mut()
            .resource_mut::<Events<SetText>>()
            .send(SetText("my sentence".to_string()));

        app.update();

        app.world_mut()
            .resource_mut::<Events<SetText>>()
            .send(SetText("".to_string()));

        app.update();

        let entities = app.world_mut().query::<Entity>().iter(&app.world()).len();

        assert_eq!(entities, 0);
    }

    #[test]
    fn should_update_sentence_text() {
        test_component_update::<Text, Sentence>("sentence", "sent", |_before, after| {
            assert_eq!(after.len(), 1);
            assert_eq!(*after[0], "sent");
        });
    }
}
