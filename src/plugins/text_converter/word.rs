use bevy::prelude::*;

use super::components::{Text, *};

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct Word;

#[derive(Bundle)]
pub struct WordBundle {
    pub name: Name,
    pub word: Word,
    pub text: Text,
    pub letters: CircleChildren,
    pub line_slots: LineSlotChildren,
}

impl WordBundle {
    pub fn new(word: String) -> Self {
        Self {
            name: Name::new("Word"),
            word: Word,
            text: Text(word),
            letters: CircleChildren::default(),
            line_slots: LineSlotChildren::default(),
        }
    }
}

pub fn convert_words(
    mut commands: Commands,
    mut sentence_query: Query<
        (Entity, &Text, &mut CircleChildren),
        (With<Sentence>, Changed<Text>),
    >,
    mut word_query: Query<(Entity, &mut Text), (With<Word>, Without<Sentence>)>,
) {
    for (sentence_entity, sentence_text, mut children) in sentence_query.iter_mut() {
        let mut existing_words = word_query.iter_many_mut(children.iter());

        let mut new_words_iter = sentence_text.split_whitespace().map(|it| it.to_string());
        let mut new_children: Vec<Entity> = Vec::new();

        loop {
            let next_exiting_word = existing_words.fetch_next();
            let next_new_word = new_words_iter.next();

            match (next_exiting_word, next_new_word) {
                // update word
                (Some((word_entity, mut word_text)), Some(new_word)) => {
                    if **word_text != new_word {
                        debug!("Update word: {} -> {}", **word_text, new_word);
                        **word_text = new_word;
                    }

                    new_children.push(word_entity);
                }
                // remove word
                (Some((word_entity, word_text)), None) => {
                    debug!("Despawn word: {}", **word_text);
                    commands.entity(word_entity).despawn_recursive();
                }
                // add word
                (None, Some(new_word)) => {
                    debug!("Spawn word: {}", new_word);

                    let bundle = WordBundle::new(new_word);

                    let word_entity = commands.spawn(bundle).id();
                    commands.entity(sentence_entity).add_child(word_entity);
                    new_children.push(word_entity);
                }
                (None, None) => {
                    break;
                }
            }
        }

        **children = new_children;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::plugins::text_converter::{
        test::test_component_update, SetText, TextConverterPlugin,
    };

    #[test]
    fn should_spawn_words() {
        test_component_update::<Text, Word>("my words", "my words", |before, _after| {
            assert_eq!(before.len(), 2);
            assert_eq!(*before[0], "my");
            assert_eq!(*before[1], "words");
        });
    }

    #[test]
    fn should_remove_word() {
        test_component_update::<Text, Word>("my words", "my", |_before, after| {
            assert_eq!(after.len(), 1);
            assert_eq!(*after[0], "my");
        });
    }

    #[test]
    fn should_despawn_children() {
        let mut app = App::new();
        app.add_plugins(TextConverterPlugin);

        let assert_occurrences =
            |app: &mut App, line_slots: usize, dots: usize, letters: usize, words: usize| {
                let line_slots_result = app
                    .world_mut()
                    .query_filtered::<Entity, With<LineSlot>>()
                    .iter(&app.world())
                    .len();
                let dots_result = app
                    .world_mut()
                    .query_filtered::<Entity, With<Dot>>()
                    .iter(&app.world())
                    .len();
                let letters_result = app
                    .world_mut()
                    .query_filtered::<Entity, With<Letter>>()
                    .iter(&app.world())
                    .len();
                let words_result = app
                    .world_mut()
                    .query_filtered::<Entity, With<Word>>()
                    .iter(&app.world())
                    .len();

                assert_eq!(line_slots_result, line_slots);
                assert_eq!(dots_result, dots);
                assert_eq!(letters_result, letters);
                assert_eq!(words_result, words);
            };

        app.world_mut()
            .resource_mut::<Events<SetText>>()
            .send(SetText("b d f".to_string()));

        app.update();

        assert_occurrences(&mut app, 3, 3, 3, 3);

        app.world_mut()
            .resource_mut::<Events<SetText>>()
            .send(SetText("b d".to_string()));

        app.update();

        assert_occurrences(&mut app, 0, 3, 2, 2);

        app.world_mut()
            .resource_mut::<Events<SetText>>()
            .send(SetText("b".to_string()));

        app.update();

        assert_occurrences(&mut app, 0, 0, 1, 1);
    }

    #[test]
    fn should_update_word_text() {
        test_component_update::<Text, Word>("my words", "me first", |_before, after| {
            assert_eq!(*after[0], "me");
            assert_eq!(*after[1], "first");
            assert_eq!(after.len(), 2);
        });
    }
}
