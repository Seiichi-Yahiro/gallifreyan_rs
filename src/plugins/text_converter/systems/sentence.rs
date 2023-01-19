use super::super::SetText;
use super::components::{Text, *};
use bevy::ecs::query::QuerySingleError;
use bevy::prelude::*;

pub fn convert_sentence(
    mut commands: Commands,
    mut events: EventReader<SetText>,
    mut sentence_query: Query<(Entity, &mut Text), With<Sentence>>,
) {
    if let Some(SetText(text)) = events.iter().last() {
        match sentence_query.get_single_mut() {
            Ok((sentence_entity, mut sentence_text)) => {
                if text.is_empty() {
                    debug!("Despawn sentence: {}", **sentence_text);
                    commands.entity(sentence_entity).despawn_recursive();
                } else {
                    debug!("Update sentence: {} -> {}", **sentence_text, text);
                    **sentence_text = text.clone();
                }
            }
            Err(QuerySingleError::NoEntities(_)) => {
                debug!("Spawn sentence: {}", text);
                let sentence_bundle = SentenceBundle::new(text.to_string());
                commands.spawn(sentence_bundle);
            }
            Err(QuerySingleError::MultipleEntities(_)) => {
                error!("Multiple sentences");
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::plugins::text_converter::test::test_component_update;
    use crate::plugins::text_converter::{SetText, TextConverterPlugin};

    #[test]
    fn should_spawn_sentence() {
        test_component_update::<Text, Sentence>(
            "my sentence",
            "my sentence",
            NestingSettings::None,
            |before, _after| {
                assert_eq!(before.len(), 1);
                assert_eq!(*before[0], "my sentence");
            },
        );
    }

    #[test]
    fn should_remove_sentence() {
        test_component_update::<Text, Sentence>(
            "my sentence",
            "",
            NestingSettings::None,
            |_before, after| {
                assert_eq!(after.len(), 0);
            },
        );
    }

    #[test]
    fn should_despawn_children() {
        let mut app = App::new();
        app.add_plugin(TextConverterPlugin);

        app.world
            .resource_mut::<Events<SetText>>()
            .send(SetText("my sentence".to_string()));

        app.update();

        app.world
            .resource_mut::<Events<SetText>>()
            .send(SetText("".to_string()));

        app.update();

        let entities = app.world.query::<Entity>().iter(&app.world).len();

        assert_eq!(entities, 0);
    }

    #[test]
    fn should_update_sentence_text() {
        test_component_update::<Text, Sentence>(
            "sentence",
            "sent",
            NestingSettings::None,
            |_before, after| {
                assert_eq!(after.len(), 1);
                assert_eq!(*after[0], "sent");
            },
        );
    }

    #[test]
    fn should_not_update_sentence_radius() {
        test_component_update::<Radius, Sentence>(
            "sentence",
            "sent",
            NestingSettings::None,
            |before, after| {
                assert_eq!(before, after);
            },
        );
    }

    #[test]
    fn should_not_update_sentence_position_data() {
        test_component_update::<PositionData, Sentence>(
            "sentence",
            "sent",
            NestingSettings::None,
            |before, after| {
                assert_eq!(before, after);
            },
        );
    }
}
