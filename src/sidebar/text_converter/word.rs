use crate::image_types::{Text, *};
use bevy::prelude::*;

pub fn convert_words(
    mut commands: Commands,
    mut sentence_query: Query<
        (Entity, &Text, &Radius, &mut CircleChildren),
        (With<Sentence>, Changed<Text>),
    >,
    mut word_query: Query<
        (Entity, &mut Text, &mut Radius, &mut PositionData),
        (With<Word>, Without<Sentence>),
    >,
) {
    for (sentence_entity, sentence_text, Radius(sentence_radius), mut children) in
        sentence_query.iter_mut()
    {
        let mut existing_words = word_query.iter_many_mut(children.iter());

        let new_words: Vec<String> = sentence_text
            .split_whitespace()
            .map(|it| it.to_string())
            .collect();
        let number_of_words = new_words.len();
        let mut new_words_iter = new_words.into_iter();

        let mut new_children: Vec<Entity> = Vec::with_capacity(number_of_words);

        loop {
            let next_exiting_word = existing_words.fetch_next();
            let next_new_word = new_words_iter.next();

            match (next_exiting_word, next_new_word) {
                // update word
                (
                    Some((word_entity, mut word_text, mut radius, mut position_data)),
                    Some(new_word),
                ) => {
                    let new_radius = Word::radius(*sentence_radius, number_of_words);

                    let new_position_data =
                        Word::position_data(*sentence_radius, number_of_words, new_children.len());

                    // TODO text change
                    //if **word_text != new_word {
                    **word_text = new_word;
                    //}

                    if **radius != new_radius {
                        **radius = new_radius;
                    }

                    if *position_data != new_position_data {
                        *position_data = new_position_data;
                    }

                    new_children.push(word_entity);
                }
                // remove word
                (Some((word_entity, _word_text, _radius, _position_data)), None) => {
                    commands.entity(word_entity).despawn_recursive();
                }
                // add word
                (None, Some(new_word)) => {
                    let word_bundle = WordBundle::new(
                        new_word,
                        *sentence_radius,
                        number_of_words,
                        new_children.len(),
                    );

                    let word_entity = commands.spawn(word_bundle).id();
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
    use super::super::test::{create_app, test_component_update};
    use super::super::SetText;
    use super::*;

    #[test]
    fn should_spawn_words() {
        test_component_update::<Text, Word>("my words", "my words", |before, _after| {
            assert_eq!(before.len(), 2);
            assert_eq!(*before[0], "my");
            assert_eq!(*before[1], "words");
        });
    }

    #[test]
    fn should_spawn_first_word_in_center() {
        test_component_update::<PositionData, Word>("my", "my", |before, _after| {
            assert_eq!(before.len(), 1);
            assert_eq!(before[0].angle.as_degrees(), 0.0);
            assert_eq!(before[0].distance, 0.0);
        });
    }

    #[test]
    fn should_spawn_move_first_word_out_of_center() {
        test_component_update::<PositionData, Word>("my", "my word", |_before, after| {
            assert_eq!(after.len(), 2);
            assert_eq!(after[0].angle.as_degrees(), 0.0);
            assert!(after[0].distance > 0.0);
            assert!(after[1].distance > 0.0);
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
        let mut app = create_app();

        let assert_occurrences =
            |app: &mut App, line_slots: usize, dots: usize, letters: usize, words: usize| {
                let line_slots_result = app
                    .world
                    .query_filtered::<Entity, With<LineSlot>>()
                    .iter(&app.world)
                    .len();
                let dots_result = app
                    .world
                    .query_filtered::<Entity, With<Dot>>()
                    .iter(&app.world)
                    .len();
                let letters_result = app
                    .world
                    .query_filtered::<Entity, With<Letter>>()
                    .iter(&app.world)
                    .len();
                let words_result = app
                    .world
                    .query_filtered::<Entity, With<Word>>()
                    .iter(&app.world)
                    .len();

                assert_eq!(line_slots_result, line_slots);
                assert_eq!(dots_result, dots);
                assert_eq!(letters_result, letters);
                assert_eq!(words_result, words);
            };

        app.world
            .resource_mut::<Events<SetText>>()
            .send(SetText("b d f".to_string()));

        app.update();

        assert_occurrences(&mut app, 3, 3, 3, 3);

        app.world
            .resource_mut::<Events<SetText>>()
            .send(SetText("b d".to_string()));

        app.update();

        assert_occurrences(&mut app, 0, 3, 2, 2);

        app.world
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

    #[test]
    fn should_update_decrease_word_radius() {
        test_component_update::<Radius, Word>("my", "my word", |before, after| {
            assert!(before[0] > after[0]);
        });
    }

    #[test]
    fn should_update_increase_word_radius() {
        test_component_update::<Radius, Word>("my word", "my", |before, after| {
            assert!(before[0] < after[0]);
        });
    }

    #[test]
    fn should_not_update_word_radius() {
        test_component_update::<Radius, Word>("word", "what", |before, after| {
            assert_eq!(before[0], after[0]);
        });
    }

    #[test]
    fn should_decrease_word_angle() {
        test_component_update::<PositionData, Word>(
            "my first",
            "my first word",
            |before, after| {
                assert_eq!(after[0].angle.as_degrees(), 0.0);
                assert!(before[1].angle > after[1].angle);
                assert!(after[0].angle < after[1].angle && after[1].angle < after[2].angle);
            },
        );
    }

    #[test]
    fn should_increase_word_angle() {
        test_component_update::<PositionData, Word>("my first word", "my word", |before, after| {
            assert_eq!(after[0].angle.as_degrees(), 0.0);
            assert!(before[1].angle < after[1].angle);
            assert!(after[0].angle < after[1].angle);
        });
    }

    #[test]
    fn should_not_update_word_angle() {
        test_component_update::<PositionData, Word>("word", "what", |before, after| {
            assert_eq!(before[0].angle, after[0].angle);
        });
    }
}
