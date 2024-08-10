mod components;
mod dot;
mod letter;
mod line_slot;
mod sentence;
mod word;

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

pub fn split_word_to_chars(word: &str) -> impl Iterator<Item = &str> {
    VALID_LETTER.find_iter(word).map(|matched| matched.as_str())
}

pub fn sanitize_text(text: &str) -> String {
    text.split_whitespace()
        .map(split_word_to_chars)
        .map(|mut word| word.join(""))
        .filter(|word| !word.is_empty())
        .join(" ")
}

pub struct TextConverterPlugin;

impl Plugin for TextConverterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Text>()
            .add_event::<SetText>()
            .add_systems(
                Update,
                (
                    set_text,
                    (
                        sentence::convert_sentence,
                        word::convert_words,
                        letter::convert_letters,
                        dot::convert_dots,
                        line_slot::convert_line_slots,
                    )
                        .chain()
                        .run_if(resource_changed::<Text>),
                )
                    .chain()
                    .run_if(on_event::<SetText>()),
            );
    }
}

#[derive(Debug, Default, Resource, Deref)]
pub struct Text(String);

#[derive(Debug, Event)]
pub struct SetText(pub String);

fn set_text(mut events: EventReader<SetText>, mut text: ResMut<Text>) {
    let Some(SetText(new_text)) = events.read().last() else {
        return;
    };

    let sanitized_text = sanitize_text(new_text);

    if text.0 == sanitized_text {
        return;
    }

    debug!("Setting sanitized text: {}", sanitized_text);
    text.0 = sanitized_text;
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
        let result: Vec<&str> = split_word_to_chars(
            "äöü+*~#'私i#あ-_.:,;<>|@n€^°1!2²\"3§³4$5v%6한글&7/{a8([9)l]0=i}ßd?\\´`",
        )
        .collect();
        let expected = ["i", "n", "v", "a", "l", "i", "d"];

        assert_eq!(result, expected);
    }

    #[test]
    fn should_sanitize_text_input() {
        let result =
            sanitize_text("äöü+*~#'私i#あ-_.:,;<>|@n€^°1!2²\"3§³4$5v%6한글&7/{a8([9)l]0=i}ßd?\\´`");
        let expected = "invalid";

        assert_eq!(result, expected);
    }

    pub fn test_component_update<C: Component + Clone, F: Component>(
        text_before: &str,
        text_after: &str,
        assert: impl Fn(Vec<C>, Vec<C>),
    ) {
        let mut app = App::new();
        app.add_plugins(TextConverterPlugin);

        app.world_mut()
            .resource_mut::<Events<SetText>>()
            .send(SetText(text_before.to_string()));

        app.update();

        let mut query = app.world_mut().query_filtered::<(Entity, &C), With<F>>();

        let before = query
            .iter(&app.world())
            .sorted_by(|(a, _), (b, _)| a.cmp(b))
            .map(|(_, c)| c.clone())
            .collect();

        app.world_mut()
            .resource_mut::<Events<SetText>>()
            .send(SetText(text_after.to_string()));

        app.update();

        let after = query
            .iter(&app.world())
            .sorted_by(|(a, _), (b, _)| a.cmp(b))
            .map(|(_, c)| c.clone())
            .collect();

        assert(before, after);
    }
}
