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
    pub static ref VOCAL: Regex = RegexBuilder::new(r"^[aeiou]$")
        .case_insensitive(true)
        .build()
        .unwrap();
}

pub struct TextConverterPlugin;

impl Plugin for TextConverterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetText>()
            .add_system(convert_sentence)
            .add_system(convert_words.after(convert_sentence))
            .add_system(convert_letters.after(convert_words))
            .add_system(convert_dots.after(convert_letters));
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
        (
            Entity,
            &mut Text,
            &mut Radius,
            &mut PositionData,
            &mut Placement,
            &mut Decoration,
        ),
        (With<Letter>, Without<Word>),
    >,
) {
    for (word_entity, word_text, Radius(word_radius), mut children) in word_query.iter_mut() {
        let mut existing_letters = letter_query.iter_many_mut(children.iter());

        let new_letters: Vec<String> = split_word_to_chars(word_text)
            .map(|it| it.to_string())
            .collect();

        let number_of_letters = new_letters.len();
        let mut new_letters_iter = new_letters.into_iter();

        let mut new_children: Vec<Entity> = Vec::with_capacity(number_of_letters);

        loop {
            let next_existing_letter = existing_letters.fetch_next();
            let next_new_letter = new_letters_iter.next();

            match (next_existing_letter, next_new_letter) {
                // update letter
                (
                    Some((
                        letter_entity,
                        mut letter_text,
                        mut radius,
                        mut position_data,
                        mut placement,
                        mut decoration,
                    )),
                    Some(new_letter),
                ) => {
                    let new_placement = Placement::try_from(new_letter.as_str()).unwrap();
                    let new_decoration = Decoration::try_from(new_letter.as_str()).unwrap();

                    let (new_radius, new_position_data) = if VOCAL.is_match(&new_letter) {
                        (
                            Vocal::radius(*word_radius, number_of_letters),
                            Vocal::position_data(
                                *word_radius,
                                number_of_letters,
                                new_children.len(),
                                new_placement,
                            ),
                        )
                    } else {
                        (
                            Consonant::radius(*word_radius, number_of_letters),
                            Consonant::position_data(
                                *word_radius,
                                number_of_letters,
                                new_children.len(),
                                new_placement,
                            ),
                        )
                    };

                    if *placement != new_placement {
                        *placement = new_placement;
                    }

                    if *decoration != new_decoration {
                        *decoration = new_decoration;
                    }

                    if **letter_text != new_letter {
                        **letter_text = new_letter;
                    }

                    if **radius != new_radius {
                        **radius = new_radius;
                    }

                    if *position_data != new_position_data {
                        *position_data = new_position_data;
                    }

                    new_children.push(letter_entity);
                }
                // remove letter
                (
                    Some((
                        letter_entity,
                        _letter_text,
                        _radius,
                        _position_data,
                        _placement,
                        _decoration,
                    )),
                    None,
                ) => {
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

fn convert_dots(
    mut commands: Commands,
    mut consonant_query: Query<
        (Entity, &Radius, &Decoration, &mut CircleChildren),
        (With<Consonant>, Changed<Text>),
    >,
    mut dot_query: Query<(Entity, &mut Radius, &mut PositionData), (With<Dot>, Without<Consonant>)>,
) {
    for (consonant_entity, Radius(consonant_radius), decoration, mut children) in
        consonant_query.iter_mut()
    {
        let mut existing_dots = dot_query.iter_many_mut(children.iter());

        let number_of_dots = decoration.dots();
        let mut new_dots_iter = 0..number_of_dots;

        let mut new_children: Vec<Entity> = Vec::with_capacity(number_of_dots);

        loop {
            let next_existing_dot = existing_dots.fetch_next();
            let next_new_dot = new_dots_iter.next();

            match (next_existing_dot, next_new_dot) {
                // update dot
                (Some((dot_entity, mut radius, mut position_data)), Some(_)) => {
                    let new_radius = Dot::radius(*consonant_radius);
                    let new_position_data =
                        Dot::position_data(*consonant_radius, number_of_dots, new_children.len());

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
                        DotBundle::new(*consonant_radius, number_of_dots, new_children.len());

                    let dot_entity = commands.spawn(dot_bundle).id();
                    commands.entity(consonant_entity).add_child(dot_entity);
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
