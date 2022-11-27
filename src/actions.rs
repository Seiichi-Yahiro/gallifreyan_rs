mod reset;

use crate::actions::reset::*;
use crate::event_set::*;
use crate::image_types::{
    CircleChildren, Consonant, ConsonantBundle, ConsonantDecoration, ConsonantPlacement, Letter,
    LineSlotChildren, PositionData, Radius, Sentence, SentenceBundle, Text, Vocal, VocalBundle,
    VocalDecoration, VocalPlacement, Word, WordBundle,
};
use crate::text_converter;
use crate::text_converter::{split_word_to_chars, VOCAL};
use bevy::ecs::query::QuerySingleError;
use bevy::prelude::*;

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event_set::<Actions>()
            .add_system(convert_sentence)
            .add_system(convert_words)
            .add_system(convert_letters);
    }
}

pub struct SetText(pub String);

event_set!(Actions { SetText, ResetAll });

fn convert_sentence(
    mut commands: Commands,
    mut events: EventReader<SetText>,
    mut sentence_query: Query<(Entity, &mut Text), With<Sentence>>,
) {
    if let Some(SetText(text)) = events.iter().last() {
        match sentence_query.get_single_mut() {
            Ok((sentence_entity, mut sentence_text)) => {
                if text.is_empty() {
                    commands.entity(sentence_entity).despawn_recursive();
                } else {
                    **sentence_text = text.clone();
                }
            }
            Err(QuerySingleError::NoEntities(_)) => {
                let sentence_bundle = SentenceBundle::new(text.to_string());
                commands.spawn(sentence_bundle);
            }
            error => {
                error.unwrap();
            }
        }
    }
}

fn convert_words(
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
        let mut new_children: Vec<Entity> = Vec::new();

        let mut existing_words = word_query.iter_many_mut(children.iter());

        let new_words: Vec<String> = sentence_text
            .split_whitespace()
            .map(|it| it.to_string())
            .collect();
        let number_of_words = new_words.len();
        let mut new_words_iter = new_words.into_iter();

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

                    if **word_text != new_word {
                        **word_text = new_word;
                    }

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

fn convert_letters(
    mut commands: Commands,
    mut word_query: Query<
        (Entity, &Text, &Radius, &mut CircleChildren),
        (With<Word>, Changed<Text>),
    >,
    mut letter_query: Query<
        (Entity, &mut Text, &mut Radius, &mut PositionData),
        (With<Letter>, Without<Word>),
    >,
) {
    for (word_entity, word_text, Radius(word_radius), mut children) in word_query.iter_mut() {
        let mut new_children: Vec<Entity> = Vec::new();

        let mut existing_letters = letter_query.iter_many_mut(children.iter());

        let new_letters: Vec<String> = split_word_to_chars(word_text)
            .map(|it| it.to_string())
            .collect();
        let number_of_letters = new_letters.len();
        let mut new_letters_iter = new_letters.into_iter();

        loop {
            let next_existing_letter = existing_letters.fetch_next();
            let next_new_letter = new_letters_iter.next();

            match (next_existing_letter, next_new_letter) {
                // update letter
                (
                    Some((letter_entity, mut letter_text, mut radius, mut position_data)),
                    Some(new_letter),
                ) => {
                    let is_vocal = VOCAL.is_match(&new_letter);

                    let new_radius = if is_vocal {
                        Vocal::radius(*word_radius, number_of_letters)
                    } else {
                        Consonant::radius(*word_radius, number_of_letters)
                    };

                    let new_position_data = if is_vocal {
                        let placement = VocalPlacement::try_from(new_letter.as_str()).unwrap();
                        Vocal::position_data(
                            *word_radius,
                            number_of_letters,
                            new_children.len(),
                            placement,
                        )
                    } else {
                        let placement = ConsonantPlacement::try_from(new_letter.as_str()).unwrap();
                        Consonant::position_data(
                            *word_radius,
                            number_of_letters,
                            new_children.len(),
                            placement,
                        )
                    };

                    if **letter_text != new_letter {
                        **letter_text = new_letter;
                    }

                    if *position_data != new_position_data {
                        *position_data = new_position_data;
                    }

                    if **radius != new_radius {
                        **radius = new_radius;
                    }

                    new_children.push(letter_entity);
                }
                // remove letter
                (Some((letter_entity, _letter_text, _radius, _position_data)), None) => {
                    commands.entity(letter_entity).despawn_recursive();
                }
                // add letter
                (None, Some(new_letter)) => {
                    let letter_entity = if VOCAL.is_match(&new_letter) {
                        let vocal_bundle = VocalBundle::new(
                            new_letter,
                            *word_radius,
                            number_of_letters,
                            new_children.len(),
                        );
                        commands.spawn(vocal_bundle).id()
                    } else {
                        let consonant_bundle = ConsonantBundle::new(
                            new_letter,
                            *word_radius,
                            number_of_letters,
                            new_children.len(),
                        );
                        commands.spawn(consonant_bundle).id()
                    };

                    commands.entity(word_entity).add_child(letter_entity);
                    new_children.push(letter_entity);
                }
                (None, None) => {
                    break;
                }
            }
        }

        **children = new_children;
    }
}
