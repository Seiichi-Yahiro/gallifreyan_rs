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

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
enum TextConverterStage {
    Sentence,
    Word,
    Letter,
    Decoration,
    Shape,
}

pub struct TextConverterPlugin;

impl Plugin for TextConverterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetText>()
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
            &mut Letter,
            &mut Text,
            &mut Radius,
            &mut PositionData,
        ),
        Without<Word>,
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
                        mut letter,
                        mut letter_text,
                        mut radius,
                        mut position_data,
                    )),
                    Some(new_letter),
                ) => {
                    *letter = Letter::try_from(new_letter.as_str()).unwrap();

                    let new_radius = letter.radius(*word_radius, number_of_letters);
                    let new_position_data =
                        letter.position_data(*word_radius, number_of_letters, new_children.len());

                    // TODO text change
                    //if **letter_text != new_letter {
                    **letter_text = new_letter;
                    //}

                    if **radius != new_radius {
                        **radius = new_radius;
                    }

                    if *position_data != new_position_data {
                        *position_data = new_position_data;
                    }

                    new_children.push(letter_entity);
                }
                // remove letter
                (Some((letter_entity, _letter, _letter_text, _radius, _position_data)), None) => {
                    commands.entity(letter_entity).despawn_recursive();
                }
                // add letter
                (None, Some(new_letter)) => {
                    let letter = Letter::try_from(new_letter.as_str()).unwrap();

                    let letter_bundle = LetterBundle::new(
                        letter,
                        new_letter,
                        *word_radius,
                        number_of_letters,
                        new_children.len(),
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

fn convert_dots(
    mut commands: Commands,
    mut letter_query: Query<(Entity, &Letter, &Radius, &mut CircleChildren), Changed<Text>>,
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
    mut letter_query: Query<(Entity, &Letter, &Radius, &mut LineSlotChildren), Changed<Text>>,
    mut line_slot_query: Query<(Entity, &mut PositionData), With<LineSlot>>,
) {
    for (letter_entity, letter, Radius(letter_radius), mut children) in letter_query.iter_mut() {
        let mut existing_line_slots = line_slot_query.iter_many_mut(children.iter());

        let number_of_lines = letter.lines();
        let line_points_outside = match letter {
            Letter::Vocal(vocal) => VocalDecoration::from(*vocal) == VocalDecoration::LineOutside,
            Letter::Consonant(_) => false,
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
    use super::*;

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

    #[test]
    fn should_sanitize_text_input() {
        let result =
            sanitize_text_input("äöü+*~#'i#-_.:,;<>|@n€^°1!2²\"3§³4$5v%6&7/{a8([9)l]0=i}ßd?\\´`");
        let expected = "invalid";

        assert_eq!(result, expected);
    }

    fn create_app() -> App {
        let mut app = App::new();

        app.add_event::<SetText>()
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
                TextConverterStage::Decoration,
                SystemStage::parallel()
                    .with_system(convert_dots)
                    .with_system(convert_line_slots),
            );

        app
    }

    fn test_component_update<C: Component + Clone, F: Component>(
        text_before: &str,
        text_after: &str,
        assert: impl Fn(Vec<C>, Vec<C>),
    ) {
        let mut app = create_app();

        app.world
            .resource_mut::<Events<SetText>>()
            .send(SetText(text_before.to_string()));

        app.update();

        let mut query = app.world.query_filtered::<&C, With<F>>();

        let before = query.iter(&app.world).cloned().collect();

        app.world
            .resource_mut::<Events<SetText>>()
            .send(SetText(text_after.to_string()));

        app.update();

        let after = query.iter(&app.world).cloned().collect();

        assert(before, after);
    }

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
    fn should_update_sentence_text() {
        test_component_update::<Text, Sentence>("sentence", "sent", |_before, after| {
            assert_eq!(after.len(), 1);
            assert_eq!(*after[0], "sent");
        });
    }

    #[test]
    fn should_not_update_sentence_radius() {
        test_component_update::<Radius, Sentence>("sentence", "sent", |before, after| {
            assert_eq!(before, after);
        });
    }

    #[test]
    fn should_not_update_sentence_position_data() {
        test_component_update::<PositionData, Sentence>("sentence", "sent", |before, after| {
            assert_eq!(before, after);
        });
    }

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

    fn test_count_letter_entities(
        text: &str,
        expected_letters: usize,
        expected_dots: usize,
        expected_line_slots: usize,
    ) {
        let mut app = create_app();

        app.world
            .resource_mut::<Events<SetText>>()
            .send(SetText(text.to_string()));

        app.update();

        let letters = app
            .world
            .query_filtered::<Entity, With<Letter>>()
            .iter(&app.world)
            .len();

        assert_eq!(letters, expected_letters);

        let dots = app
            .world
            .query_filtered::<Entity, With<Dot>>()
            .iter(&app.world)
            .len();

        assert_eq!(dots, expected_dots);

        let line_slots = app
            .world
            .query_filtered::<Entity, With<LineSlot>>()
            .iter(&app.world)
            .len();

        assert_eq!(line_slots, expected_line_slots);
    }

    #[test]
    fn should_spawn_b() {
        test_count_letter_entities("b", 1, 0, 0);
    }

    #[test]
    fn should_spawn_j() {
        test_count_letter_entities("j", 1, 0, 0);
    }

    #[test]
    fn should_spawn_t() {
        test_count_letter_entities("t", 1, 0, 0);
    }

    #[test]
    fn should_spawn_th() {
        test_count_letter_entities("th", 1, 0, 0);
    }

    #[test]
    fn should_spawn_ph() {
        test_count_letter_entities("ph", 1, 1, 0);
    }

    #[test]
    fn should_spawn_wh() {
        test_count_letter_entities("wh", 1, 1, 0);
    }

    #[test]
    fn should_spawn_gh() {
        test_count_letter_entities("gh", 1, 1, 0);
    }

    #[test]
    fn should_spawn_ch() {
        test_count_letter_entities("ch", 1, 2, 0);
    }

    #[test]
    fn should_spawn_k() {
        test_count_letter_entities("k", 1, 2, 0);
    }

    #[test]
    fn should_spawn_sh() {
        test_count_letter_entities("sh", 1, 2, 0);
    }

    #[test]
    fn should_spawn_y() {
        test_count_letter_entities("y", 1, 2, 0);
    }

    #[test]
    fn should_spawn_d() {
        test_count_letter_entities("d", 1, 3, 0);
    }

    #[test]
    fn should_spawn_l() {
        test_count_letter_entities("l", 1, 3, 0);
    }

    #[test]
    fn should_spawn_r() {
        test_count_letter_entities("r", 1, 3, 0);
    }

    #[test]
    fn should_spawn_z() {
        test_count_letter_entities("z", 1, 3, 0);
    }

    #[test]
    fn should_spawn_c() {
        test_count_letter_entities("c", 1, 4, 0);
    }

    #[test]
    fn should_spawn_q() {
        test_count_letter_entities("q", 1, 4, 0);
    }

    #[test]
    fn should_spawn_g() {
        test_count_letter_entities("g", 1, 0, 1);
    }

    #[test]
    fn should_spawn_n() {
        test_count_letter_entities("n", 1, 0, 1);
    }

    #[test]
    fn should_spawn_v() {
        test_count_letter_entities("v", 1, 0, 1);
    }

    #[test]
    fn should_spawn_qu() {
        test_count_letter_entities("qu", 1, 0, 1);
    }

    #[test]
    fn should_spawn_h() {
        test_count_letter_entities("h", 1, 0, 2);
    }

    #[test]
    fn should_spawn_p() {
        test_count_letter_entities("p", 1, 0, 2);
    }

    #[test]
    fn should_spawn_w() {
        test_count_letter_entities("w", 1, 0, 2);
    }

    #[test]
    fn should_spawn_x() {
        test_count_letter_entities("x", 1, 0, 2);
    }

    #[test]
    fn should_spawn_f() {
        test_count_letter_entities("f", 1, 0, 3);
    }

    #[test]
    fn should_spawn_m() {
        test_count_letter_entities("m", 1, 0, 3);
    }

    #[test]
    fn should_spawn_s() {
        test_count_letter_entities("s", 1, 0, 3);
    }

    #[test]
    fn should_spawn_ng() {
        test_count_letter_entities("ng", 1, 0, 3);
    }

    #[test]
    fn should_spawn_a() {
        test_count_letter_entities("a", 1, 0, 0);
    }

    #[test]
    fn should_spawn_e() {
        test_count_letter_entities("e", 1, 0, 0);
    }

    #[test]
    fn should_spawn_i() {
        test_count_letter_entities("i", 1, 0, 1);
    }

    #[test]
    fn should_spawn_o() {
        test_count_letter_entities("o", 1, 0, 0);
    }

    #[test]
    fn should_spawn_u() {
        test_count_letter_entities("u", 1, 0, 1);
    }

    #[test]
    fn should_decrease_letter_radius() {
        test_component_update::<Radius, Letter>("jj", "jjj", |before, after| {
            assert!(before[1] > after[1]);
        });
    }

    #[test]
    fn should_increase_letter_radius() {
        test_component_update::<Radius, Letter>("jjj", "jj", |before, after| {
            assert!(before[1] < after[1]);
        });
    }

    #[test]
    fn should_not_update_letter_radius() {
        test_component_update::<Radius, Letter>("jjj", "jjj", |before, after| {
            assert_eq!(before[1], after[1]);
        });
    }

    #[test]
    fn should_decrease_letter_angle() {
        test_component_update::<PositionData, Letter>("jj", "jjj", |before, after| {
            assert_eq!(after[0].angle.as_degrees(), 0.0);
            assert!(before[1].angle > after[1].angle);
            assert!(after[0].angle < after[1].angle && after[1].angle < after[2].angle);
        });
    }

    #[test]
    fn should_increase_letter_angle() {
        test_component_update::<PositionData, Letter>("jjj", "jj", |before, after| {
            assert_eq!(after[0].angle.as_degrees(), 0.0);
            assert!(before[1].angle < after[1].angle);
            assert!(after[0].angle < after[1].angle);
        });
    }

    #[test]
    fn should_not_update_letter_angle() {
        test_component_update::<PositionData, Letter>("jjj", "jjj", |before, after| {
            assert_eq!(before[1].angle, after[1].angle);
        });
    }
}
