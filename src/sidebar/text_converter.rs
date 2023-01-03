mod dot;
mod letter;
mod line_slot;
mod sentence;
mod word;

use crate::image_types::{
    add_shape_for_dot, add_shape_for_letter, add_shape_for_line_slot, add_shape_for_sentence,
    add_shape_for_word, NestingSettings,
};
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
                SystemStage::single(sentence::convert_sentence),
            )
            .add_stage_after(
                TextConverterStage::Sentence,
                TextConverterStage::Word,
                SystemStage::single(word::convert_words),
            )
            .add_stage_after(
                TextConverterStage::Word,
                TextConverterStage::Letter,
                SystemStage::single(letter::convert_letters),
            )
            .add_stage_after(
                TextConverterStage::Letter,
                TextConverterStage::Nested,
                SystemStage::single(letter::convert_nested_letters),
            )
            .add_stage_after(
                TextConverterStage::Nested,
                TextConverterStage::Decoration,
                SystemStage::parallel()
                    .with_system(dot::convert_dots)
                    .with_system(line_slot::convert_line_slots),
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

    pub fn create_app() -> App {
        let mut app = App::new();

        app.add_event::<SetText>()
            .insert_resource(NestingSettings::None)
            .add_stage_before(
                CoreStage::Update,
                TextConverterStage::Sentence,
                SystemStage::single(sentence::convert_sentence),
            )
            .add_stage_after(
                TextConverterStage::Sentence,
                TextConverterStage::Word,
                SystemStage::single(word::convert_words),
            )
            .add_stage_after(
                TextConverterStage::Word,
                TextConverterStage::Letter,
                SystemStage::single(letter::convert_letters),
            )
            .add_stage_after(
                TextConverterStage::Letter,
                TextConverterStage::Decoration,
                SystemStage::parallel()
                    .with_system(dot::convert_dots)
                    .with_system(line_slot::convert_line_slots),
            );

        app
    }

    pub fn test_component_update<C: Component + Clone, F: Component>(
        text_before: &str,
        text_after: &str,
        assert: impl Fn(Vec<C>, Vec<C>),
    ) {
        let mut app = create_app();

        app.world
            .resource_mut::<Events<SetText>>()
            .send(SetText(text_before.to_string()));

        app.update();

        let mut query = app.world.query_filtered::<(Entity, &C), With<F>>();

        let before = query
            .iter(&app.world)
            .sorted_by(|(a, _), (b, _)| a.cmp(b))
            .map(|(_, c)| c.clone())
            .collect();

        app.world
            .resource_mut::<Events<SetText>>()
            .send(SetText(text_after.to_string()));

        app.update();

        let after = query
            .iter(&app.world)
            .sorted_by(|(a, _), (b, _)| a.cmp(b))
            .map(|(_, c)| c.clone())
            .collect();

        assert(before, after);
    }
}
