pub mod components;
pub mod systems;

use bevy::prelude::*;
use components::NestingSettings;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref VALID_LETTER: Regex = RegexBuilder::new(r"[cpwstg]h?|ng?|qu?|[aeioubdhfjklmrvyzx]")
        .case_insensitive(true)
        .build()
        .unwrap();
}

#[derive(SystemSet, Debug, Eq, PartialEq, Copy, Clone, Hash)]
#[system_set(base)]
pub enum TextConverterBaseSet {
    TextConverter,
    PostTextConverter,
    PostTextConverterFlush,
}

pub struct TextConverterPlugin;

impl Plugin for TextConverterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetText>()
            .insert_resource(NestingSettings::All)
            .configure_sets(
                (
                    TextConverterBaseSet::TextConverter,
                    TextConverterBaseSet::PostTextConverter,
                    TextConverterBaseSet::PostTextConverterFlush,
                )
                    .chain()
                    .before(CoreSet::Update),
            )
            .add_systems(
                (
                    systems::sentence::convert_sentence,
                    apply_system_buffers,
                    systems::word::convert_words,
                    apply_system_buffers,
                    systems::letter::convert_letters,
                    apply_system_buffers,
                    systems::letter::convert_nested_letters,
                    apply_system_buffers,
                    systems::dot::convert_dots,
                    systems::line_slot::convert_line_slots,
                    apply_system_buffers,
                )
                    .chain()
                    .in_base_set(TextConverterBaseSet::TextConverter),
            )
            .add_system(
                apply_system_buffers.in_base_set(TextConverterBaseSet::PostTextConverterFlush),
            )
            .register_type::<components::Sentence>()
            .register_type::<components::Word>()
            .register_type::<components::Letter>()
            .register_type::<components::Consonant>()
            .register_type::<components::Vocal>()
            .register_type::<components::NestedLetter>()
            .register_type::<components::NestedVocal>()
            .register_type::<components::NestedVocalPositionCorrection>()
            .register_type::<Option<Entity>>()
            .register_type::<components::Dot>()
            .register_type::<components::LineSlot>()
            .register_type::<components::CircleChildren>()
            .register_type::<components::LineSlotChildren>()
            .register_type::<components::Text>()
            .register_type::<components::Radius>()
            .register_type::<components::PositionData>()
            .register_type::<components::AnglePlacement>();
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

    pub fn test_component_update<C: Component + Clone, F: Component>(
        text_before: &str,
        text_after: &str,
        nesting_settings: NestingSettings,
        assert: impl Fn(Vec<C>, Vec<C>),
    ) {
        let mut app = App::new();
        app.add_plugin(TextConverterPlugin)
            .insert_resource(nesting_settings);

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
