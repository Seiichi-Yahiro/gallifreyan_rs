use crate::image_types::Text;
use crate::image_types::*;
use bevy::ecs::query::QuerySingleError;
use bevy::prelude::*;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref VALID_LETTER: Regex = RegexBuilder::new(r"[cpwstg]h?|ng?|qu?|[aeioubdhfjklmrvyzx]")
        .case_insensitive(true)
        .build()
        .unwrap();
}

const NESTED_LETTER_TEXT_DELIMITER: &str = "~";

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
enum TextConverterStage {
    Sentence,
    Word,
    Letter,
    Nested,
    Decoration,
    Shape,
}

pub struct TextConverterPlugin;

impl Plugin for TextConverterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetText>()
            .insert_resource(NestingSettings::All)
            .add_stage_before(
                CoreStage::Update,
                TextConverterStage::Sentence,
                SystemStage::single(convert_sentence),
            )
            .add_stage_after(
                TextConverterStage::Sentence,
                TextConverterStage::Word,
                SystemStage::single(convert_words),
            )
            .add_stage_after(
                TextConverterStage::Word,
                TextConverterStage::Letter,
                SystemStage::single(convert_letters),
            )
            .add_stage_after(
                TextConverterStage::Letter,
                TextConverterStage::Nested,
                SystemStage::single(convert_nested_letters),
            )
            .add_stage_after(
                TextConverterStage::Nested,
                TextConverterStage::Decoration,
                SystemStage::parallel()
                    .with_system(convert_dots)
                    .with_system(convert_line_slots),
            )
            .add_stage_after(
                TextConverterStage::Decoration,
                TextConverterStage::Shape,
                SystemStage::parallel()
                    .with_system(add_shape_for_sentence)
                    .with_system(add_shape_for_word)
                    .with_system(add_shape_for_letter)
                    .with_system(add_shape_for_dot)
                    .with_system(add_shape_for_line_slot),
            );
    }
}

pub struct SetText(pub String);

pub fn split_word_to_chars(word: &str) -> impl Iterator<Item = &str> {
    VALID_LETTER.find_iter(word).map(|matched| matched.as_str())
}

pub fn sanitize_text_input(text: &str) -> String {
    text.split_whitespace()
        .map(split_word_to_chars)
        .map(|mut word| word.join(""))
        .filter(|word| !word.is_empty())
        .join(" ")
}

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

fn convert_letters(
    mut commands: Commands,
    mut word_query: Query<
        (Entity, &Text, &Radius, &mut CircleChildren),
        (With<Word>, Changed<Text>),
    >,
    mut letter_query: Query<
        (
            Entity,
            &mut Text,
            &mut Letter,
            &mut Radius,
            &mut PositionData,
        ),
        (Without<Word>, Without<NestedVocal>),
    >,
    nesting_settings: Res<NestingSettings>,
) {
    for (word_entity, word_text, Radius(word_radius), mut children) in word_query.iter_mut() {
        let mut existing_letters = letter_query.iter_many_mut(children.iter());

        let new_letters: Vec<(String, Letter)> = split_word_to_chars(word_text)
            .map(|it| {
                let letter = Letter::try_from(it).unwrap();
                (it.to_string(), letter)
            })
            .fold(Vec::new(), |mut acc, (text, letter)| {
                match nesting_settings.as_ref() {
                    NestingSettings::None => {
                        acc.push((text, letter));
                    }
                    nesting_settings => match letter {
                        Letter::Vocal(vocal) => {
                            if let Some((previous_text, previous_letter)) = acc.pop() {
                                match previous_letter {
                                    Letter::Consonant(consonant)
                                        if nesting_settings.can_nest(consonant, vocal) =>
                                    {
                                        acc.push((
                                            previous_text + NESTED_LETTER_TEXT_DELIMITER + &text,
                                            Letter::ConsonantWithVocal { consonant, vocal },
                                        ));
                                    }
                                    _ => {
                                        acc.push((previous_text, previous_letter));
                                        acc.push((text, letter));
                                    }
                                }
                            } else {
                                acc.push((text, letter));
                            }
                        }
                        Letter::Consonant(_) | Letter::ConsonantWithVocal { .. } => {
                            acc.push((text, letter));
                        }
                    },
                }

                acc
            });

        let number_of_letters = new_letters.len();
        let mut new_letters_iter = new_letters.into_iter();

        let mut new_children: Vec<Entity> = Vec::with_capacity(number_of_letters);

        loop {
            let next_existing_letter = existing_letters.fetch_next();
            let next_new_letter = new_letters_iter.next();

            match (next_existing_letter, next_new_letter) {
                // update letter
                (
                    Some((letter_entity, mut text, mut letter, mut radius, mut position_data)),
                    Some((new_text, new_letter)),
                ) => {
                    let new_radius = new_letter.radius(*word_radius, number_of_letters);
                    let new_position_data = new_letter.position_data(
                        *word_radius,
                        number_of_letters,
                        new_children.len(),
                    );

                    **text = new_text;
                    *letter = new_letter;

                    if **radius != new_radius {
                        **radius = new_radius;
                    }

                    if *position_data != new_position_data {
                        *position_data = new_position_data;
                    }

                    new_children.push(letter_entity);
                }
                // remove letter
                (Some((letter_entity, _text, _letter, _radius, _position_data)), None) => {
                    commands.entity(letter_entity).despawn_recursive();
                }
                // add letter
                (None, Some((text, new_letter))) => {
                    let letter_bundle = LetterBundle::new(
                        text,
                        new_letter,
                        *word_radius,
                        number_of_letters,
                        new_children.len(),
                        None,
                    );

                    let letter_entity = commands.spawn(letter_bundle).id();
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

fn convert_nested_letters(
    mut commands: Commands,
    word_query: Query<&Radius, (With<Word>, Without<NestedVocal>)>,
    mut letter_query: Query<
        (
            Entity,
            &Parent,
            &Text,
            &Letter,
            &PositionData,
            &Radius,
            &mut NestedLetter,
        ),
        (Changed<Letter>, Without<NestedVocal>),
    >,
    position_correction_query: Query<Entity, With<NestedVocalPositionCorrection>>,
    mut nested_vocal_query: Query<
        (
            &Parent,
            &mut Text,
            &mut Letter,
            &mut Radius,
            &mut PositionData,
        ),
        With<NestedVocal>,
    >,
) {
    for (
        letter_entity,
        letter_parent,
        letter_text,
        letter,
        letter_position_data,
        letter_radius,
        mut nested,
    ) in letter_query.iter_mut()
    {
        match letter {
            Letter::ConsonantWithVocal { vocal, consonant } => {
                let word_radius = **word_query.get(letter_parent.get()).unwrap();
                let new_nested_text = letter_text
                    .split_once(NESTED_LETTER_TEXT_DELIMITER)
                    .unwrap()
                    .1
                    .to_string();

                if let Some(nested_entity) = nested.take() {
                    // update nested
                    if let Ok((
                        nested_parent,
                        mut nested_text,
                        mut nested_letter,
                        mut nested_radius,
                        mut nested_position_data,
                    )) = nested_vocal_query.get_mut(nested_entity)
                    {
                        let old_placement = match *nested_letter {
                            Letter::Vocal(vocal) => VocalPlacement::from(vocal),
                            _ => unreachable!(),
                        };

                        let new_placement = VocalPlacement::from(*vocal);

                        match (old_placement, new_placement) {
                            // add position correction
                            (
                                VocalPlacement::OnLine | VocalPlacement::Inside,
                                VocalPlacement::Outside,
                            ) => {
                                commands
                                    .entity(letter_entity)
                                    .remove_children(&[nested_entity])
                                    .with_children(|child_builder| {
                                        child_builder
                                            .spawn(NestedVocalPositionCorrectionBundle::new(
                                                letter_position_data.distance,
                                            ))
                                            .add_child(nested_entity);
                                    });
                            }
                            // remove position correction
                            (
                                VocalPlacement::Outside,
                                VocalPlacement::OnLine | VocalPlacement::Inside,
                            ) => {
                                commands
                                    .entity(nested_parent.get())
                                    .remove_children(&[nested_entity])
                                    .despawn();
                                commands.entity(letter_entity).add_child(nested_entity);
                            }
                            _ => {}
                        }

                        **nested_text = new_nested_text;
                        *nested_letter = Letter::Vocal(*vocal);

                        let new_nested_radius = vocal.nested_radius(**letter_radius);
                        let new_nested_position_data = vocal.nested_position_data(
                            ConsonantPlacement::from(*consonant),
                            **letter_radius,
                            letter_position_data.distance,
                            word_radius,
                        );

                        if **nested_radius != new_nested_radius {
                            **nested_radius = new_nested_radius;
                        }

                        if *nested_position_data != new_nested_position_data {
                            *nested_position_data = new_nested_position_data;
                        }
                    }

                    **nested = Some(nested_entity);
                } else {
                    // spawn nested
                    let vocal_id = commands
                        .entity(letter_entity)
                        .add_children(|child_builder| {
                            let vocal_bundle = NestedVocalBundle::new(
                                new_nested_text,
                                *vocal,
                                ConsonantPlacement::from(*consonant),
                                **letter_radius,
                                letter_position_data.distance,
                                word_radius,
                            );

                            if VocalPlacement::Outside == VocalPlacement::from(*vocal) {
                                child_builder
                                    .spawn(NestedVocalPositionCorrectionBundle::new(
                                        letter_position_data.distance,
                                    ))
                                    .add_children(|child_builder| {
                                        child_builder.spawn(vocal_bundle).id()
                                    })
                            } else {
                                child_builder.spawn(vocal_bundle).id()
                            }
                        });

                    **nested = Some(vocal_id);
                }
            }
            Letter::Consonant(_) | Letter::Vocal(_) => {
                // remove nested
                if let Some(nested_entity) = nested.take() {
                    let position_correction_entity = nested_vocal_query
                        .get(nested_entity)
                        .and_then(|(parent, _, _, _, _)| {
                            position_correction_query.get(parent.get())
                        });

                    if let Ok(position_correction_entity) = position_correction_entity {
                        commands
                            .entity(position_correction_entity)
                            .despawn_recursive();
                    } else {
                        commands.entity(nested_entity).despawn_recursive();
                    }
                }
            }
        }
    }
}

fn convert_dots(
    mut commands: Commands,
    mut letter_query: Query<(Entity, &Letter, &Radius, &mut CircleChildren), Changed<Letter>>,
    mut dot_query: Query<(Entity, &mut Radius, &mut PositionData), (With<Dot>, Without<Letter>)>,
) {
    for (letter_entity, letter, Radius(letter_radius), mut children) in letter_query.iter_mut() {
        let mut existing_dots = dot_query.iter_many_mut(children.iter());

        let number_of_dots = letter.dots();
        let mut new_dots_iter = 0..number_of_dots;

        let mut new_children: Vec<Entity> = Vec::with_capacity(number_of_dots);

        loop {
            let next_existing_dot = existing_dots.fetch_next();
            let next_new_dot = new_dots_iter.next();

            match (next_existing_dot, next_new_dot) {
                // update dot
                (Some((dot_entity, mut radius, mut position_data)), Some(_)) => {
                    let new_radius = Dot::radius(*letter_radius);
                    let new_position_data =
                        Dot::position_data(*letter_radius, number_of_dots, new_children.len());

                    if **radius != new_radius {
                        **radius = new_radius;
                    }

                    if *position_data != new_position_data {
                        *position_data = new_position_data;
                    }

                    new_children.push(dot_entity);
                }
                // remove dot
                (Some((dot_entity, _radius, _position_data)), None) => {
                    commands.entity(dot_entity).despawn_recursive();
                }
                // add dot
                (None, Some(_)) => {
                    let dot_bundle =
                        DotBundle::new(*letter_radius, number_of_dots, new_children.len());

                    let dot_entity = commands.spawn(dot_bundle).id();
                    commands.entity(letter_entity).add_child(dot_entity);
                    new_children.push(dot_entity);
                }
                (None, None) => {
                    break;
                }
            }
        }

        **children = new_children;
    }
}

fn convert_line_slots(
    mut commands: Commands,
    mut letter_query: Query<(Entity, &Letter, &Radius, &mut LineSlotChildren), Changed<Letter>>,
    mut line_slot_query: Query<(Entity, &mut PositionData), With<LineSlot>>,
) {
    for (letter_entity, letter, Radius(letter_radius), mut children) in letter_query.iter_mut() {
        let mut existing_line_slots = line_slot_query.iter_many_mut(children.iter());

        let number_of_lines = letter.lines();
        let line_points_outside = match letter {
            Letter::Vocal(vocal) => VocalDecoration::from(*vocal) == VocalDecoration::LineOutside,
            Letter::Consonant(_) | Letter::ConsonantWithVocal { .. } => false,
        };
        let mut new_line_slots_iter = 0..number_of_lines;

        let mut new_children: Vec<Entity> = Vec::with_capacity(number_of_lines);

        loop {
            let next_existing_line_slot = existing_line_slots.fetch_next();
            let next_new_line_slot = new_line_slots_iter.next();

            match (next_existing_line_slot, next_new_line_slot) {
                // update line slot
                (Some((line_slot_entity, mut position_data)), Some(_)) => {
                    let new_position_data = LineSlot::position_data(
                        *letter_radius,
                        number_of_lines,
                        new_children.len(),
                        line_points_outside,
                    );

                    if *position_data != new_position_data {
                        *position_data = new_position_data;
                    }

                    new_children.push(line_slot_entity);
                }
                // remove line slot
                (Some((line_slot_entity, _position_data)), None) => {
                    commands.entity(line_slot_entity).despawn_recursive();
                }
                // add line slot
                (None, Some(_)) => {
                    let line_slot_bundle = LineSlotBundle::new(
                        *letter_radius,
                        number_of_lines,
                        new_children.len(),
                        line_points_outside,
                    );

                    let line_slot_entity = commands.spawn(line_slot_bundle).id();
                    commands.entity(letter_entity).add_child(line_slot_entity);
                    new_children.push(line_slot_entity);
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
    use super::split_word_to_chars;

    #[test]
    fn should_split_lower_case_word() {
        let result: Vec<&str> =
            split_word_to_chars("aeioubjtthphwhghchkshydlrzcqgnvquhpwxfmsng").collect();
        let expected = [
            "a", "e", "i", "o", "u", "b", "j", "t", "th", "ph", "wh", "gh", "ch", "k", "sh", "y",
            "d", "l", "r", "z", "c", "q", "g", "n", "v", "qu", "h", "p", "w", "x", "f", "m", "s",
            "ng",
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn should_split_upper_case_word() {
        let result: Vec<&str> =
            split_word_to_chars("AEIOUBJTTHPHWHGHCHKSHYDLRZCQGNVQUHPWXFMSNG").collect();
        let expected = [
            "A", "E", "I", "O", "U", "B", "J", "T", "TH", "PH", "WH", "GH", "CH", "K", "SH", "Y",
            "D", "L", "R", "Z", "C", "Q", "G", "N", "V", "QU", "H", "P", "W", "X", "F", "M", "S",
            "NG",
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn should_split_mixed_case_double_letters() {
        let result: Vec<&str> = split_word_to_chars("tHThpHPhwHWhgHGhcHChsHShqUQunGNg").collect();
        let expected = [
            "tH", "Th", "pH", "Ph", "wH", "Wh", "gH", "Gh", "cH", "Ch", "sH", "Sh", "qU", "Qu",
            "nG", "Ng",
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn should_filter_invalid_letters() {
        let result: Vec<&str> =
            split_word_to_chars("äöü+*~#'i#-_.:,;<>|@n€^°1!2²\"3§³4$5v%6&7/{a8([9)l]0=i}ßd?\\´`")
                .collect();
        let expected = ["i", "n", "v", "a", "l", "i", "d"];

        assert_eq!(result, expected);
    }
}
