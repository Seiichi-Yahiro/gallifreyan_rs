mod letter;

use crate::plugins::text_converter::letter::combinator::CombineLetters;
use crate::plugins::text_converter::letter::Letter;
use bevy::prelude::*;
use itertools::Itertools;
use similar::{Algorithm, ChangeTag, TextDiff};
use unicode_segmentation::UnicodeSegmentation;

pub struct TextConverterPlugin;

impl Plugin for TextConverterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SentenceText>()
            .add_event::<SetText>()
            .add_event::<TextModification>()
            .add_systems(
                Update,
                set_text.in_set(SetTextSet).run_if(on_event::<SetText>()),
            );

        #[cfg(debug_assertions)]
        app.add_systems(
            Update,
            debug_text_modifications
                .after(set_text)
                .in_set(SetTextSet)
                .run_if(on_event::<TextModification>()),
        );
    }
}

fn debug_text_modifications(mut events: EventReader<TextModification>) {
    for event in events.read() {
        debug!("{:?}", event);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct SetTextSet;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub struct LetterId {
    pub sentence: usize,
    pub word: usize,
    pub letter: usize,
}

#[cfg(test)]
impl From<(usize, usize, usize)> for LetterId {
    fn from(value: (usize, usize, usize)) -> Self {
        Self {
            sentence: value.0,
            word: value.1,
            letter: value.2,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Resource)]
pub struct SentenceText {
    pub local_index: usize,
    pub text: String,
    pub words: Vec<WordText>,
}

impl SentenceText {
    fn flatten_letters(&self) -> (Vec<LetterId>, Vec<&str>) {
        self.words
            .iter()
            .flat_map(|word| {
                word.letters
                    .iter()
                    .map(|letter| {
                        (
                            LetterId {
                                sentence: self.local_index,
                                word: word.local_index,
                                letter: letter.local_index,
                            },
                            letter.text.as_str(),
                        )
                    })
                    .chain(std::iter::once((
                        LetterId {
                            // this is a fake id that will not be used
                            sentence: 0,
                            word: 0,
                            letter: 0,
                        },
                        " ", // add space for diff algo
                    )))
            })
            .unzip()
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct WordText {
    pub local_index: usize,
    pub text: String,
    pub letters: Vec<LetterText>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LetterText {
    pub local_index: usize,
    pub text: String,
    pub letter: Letter,
}

#[cfg(test)]
impl LetterText {
    fn new<T: Into<String>, L: Into<Letter>>(local_index: usize, text: T, letter: L) -> Self {
        Self {
            local_index,
            text: text.into(),
            letter: letter.into(),
        }
    }
}

fn split_word_to_letters(word: &str) -> impl Iterator<Item=LetterText> + '_ {
    word.graphemes(true)
        .filter_map(|grapheme| {
            Letter::try_from(grapheme)
                .ok()
                .map(|letter| (grapheme, letter))
        })
        .enumerate()
        .map(|(local_index, (grapheme, letter))| LetterText {
            local_index,
            text: grapheme.to_string(),
            letter,
        })
        .combine()
}

fn convert_sentence(sentence: &str) -> SentenceText {
    let words = sentence
        .split_whitespace()
        .map(split_word_to_letters)
        .map(Vec::from_iter)
        .filter(|letters| !letters.is_empty())
        .enumerate()
        .map(|(local_index, letters)| WordText {
            local_index,
            text: letters.iter().map(|letter| letter.text.as_str()).join(""),
            letters,
        })
        .collect_vec();

    SentenceText {
        local_index: 0,
        text: words.iter().map(|word| word.text.as_str()).join(" "),
        words,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Event)]
pub enum TextModification {
    Create {
        new_id: LetterId,
        text: String,
    },
    Move {
        old_id: LetterId,
        new_id: LetterId,
        text: String,
    },
    Delete {
        old_id: LetterId,
        text: String,
    },
}

#[cfg(test)]
impl TextModification {
    fn create<I: Into<LetterId>, T: Into<String>>(new_id: I, text: T) -> Self {
        Self::Create {
            new_id: new_id.into(),
            text: text.into(),
        }
    }

    fn r#move<I: Into<LetterId>, T: Into<String>>(old_id: I, new_id: I, text: T) -> Self {
        Self::Move {
            old_id: old_id.into(),
            new_id: new_id.into(),
            text: text.into(),
        }
    }

    fn delete<I: Into<LetterId>, T: Into<String>>(old_id: I, text: T) -> Self {
        Self::Delete {
            old_id: old_id.into(),
            text: text.into(),
        }
    }
}

#[derive(Debug, Event)]
pub struct SetText(pub String);

fn set_text(
    mut events: EventReader<SetText>,
    mut sentence_text: ResMut<SentenceText>,
    mut text_modifications_events: EventWriter<TextModification>,
) {
    let Some(SetText(new_sentence)) = events.read().last() else {
        return;
    };

    let new_sentence = convert_sentence(new_sentence);
    let old_sentence = std::mem::replace(&mut *sentence_text, new_sentence);
    let new_sentence = &*sentence_text;

    let (old_letter_ids, old_letters) = old_sentence.flatten_letters();
    let (new_letter_ids, new_letters) = new_sentence.flatten_letters();

    let diff = TextDiff::configure()
        .algorithm(Algorithm::Myers)
        .diff_slices(&old_letters, &new_letters);

    let text_modifications = diff
        .iter_all_changes()
        .filter(|change| change.value() != " ")
        .map(|change| match change.tag() {
            ChangeTag::Equal => TextModification::Move {
                old_id: old_letter_ids[change.old_index().unwrap()],
                new_id: new_letter_ids[change.new_index().unwrap()],
                text: change.value().to_string(),
            },
            ChangeTag::Delete => TextModification::Delete {
                old_id: old_letter_ids[change.old_index().unwrap()],
                text: change.value().to_string(),
            },
            ChangeTag::Insert => TextModification::Create {
                new_id: new_letter_ids[change.new_index().unwrap()],
                text: change.value().to_string(),
            },
        })
        .filter(|modification| match modification {
            TextModification::Move { old_id, new_id, .. } if old_id == new_id => false,
            _ => true,
        });

    text_modifications_events.send_batch(text_modifications);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::plugins::text_converter::letter::consonant::{Consonant, Digraph};
    use crate::plugins::text_converter::letter::vocal::Vocal;
    use bevy::ecs::event::ManualEventReader;

    #[test]
    fn should_split_lower_case_word() {
        let result: Vec<LetterText> =
            split_word_to_letters("aeioubjtthphwhghchkshydlrzcqgnvquhpwxfmsng").collect();

        let expected = [
            LetterText::new(0, "a", Vocal::A),
            LetterText::new(1, "e", Vocal::E),
            LetterText::new(2, "i", Vocal::I),
            LetterText::new(3, "o", Vocal::O),
            LetterText::new(4, "u", Vocal::U),
            LetterText::new(5, "b", Consonant::B),
            LetterText::new(6, "j", Consonant::J),
            LetterText::new(7, "t", Consonant::T),
            LetterText::new(8, "th", Digraph::TH),
            LetterText::new(9, "ph", Digraph::PH),
            LetterText::new(10, "wh", Digraph::WH),
            LetterText::new(11, "gh", Digraph::GH),
            LetterText::new(12, "ch", Digraph::CH),
            LetterText::new(13, "k", Consonant::K),
            LetterText::new(14, "sh", Digraph::SH),
            LetterText::new(15, "y", Consonant::Y),
            LetterText::new(16, "d", Consonant::D),
            LetterText::new(17, "l", Consonant::L),
            LetterText::new(18, "r", Consonant::R),
            LetterText::new(19, "z", Consonant::Z),
            LetterText::new(20, "c", Consonant::C),
            LetterText::new(21, "q", Consonant::Q),
            LetterText::new(22, "g", Consonant::G),
            LetterText::new(23, "n", Consonant::N),
            LetterText::new(24, "v", Consonant::V),
            LetterText::new(25, "qu", Digraph::QU),
            LetterText::new(26, "h", Consonant::H),
            LetterText::new(27, "p", Consonant::P),
            LetterText::new(28, "w", Consonant::W),
            LetterText::new(29, "x", Consonant::X),
            LetterText::new(30, "f", Consonant::F),
            LetterText::new(31, "m", Consonant::M),
            LetterText::new(32, "s", Consonant::S),
            LetterText::new(33, "ng", Digraph::NG),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn should_split_upper_case_word() {
        let result: Vec<LetterText> =
            split_word_to_letters("AEIOUBJTTHPHWHGHCHKSHYDLRZCQGNVQUHPWXFMSNG").collect();

        let expected = [
            LetterText::new(0, "A", Vocal::A),
            LetterText::new(1, "E", Vocal::E),
            LetterText::new(2, "I", Vocal::I),
            LetterText::new(3, "O", Vocal::O),
            LetterText::new(4, "U", Vocal::U),
            LetterText::new(5, "B", Consonant::B),
            LetterText::new(6, "J", Consonant::J),
            LetterText::new(7, "T", Consonant::T),
            LetterText::new(8, "TH", Digraph::TH),
            LetterText::new(9, "PH", Digraph::PH),
            LetterText::new(10, "WH", Digraph::WH),
            LetterText::new(11, "GH", Digraph::GH),
            LetterText::new(12, "CH", Digraph::CH),
            LetterText::new(13, "K", Consonant::K),
            LetterText::new(14, "SH", Digraph::SH),
            LetterText::new(15, "Y", Consonant::Y),
            LetterText::new(16, "D", Consonant::D),
            LetterText::new(17, "L", Consonant::L),
            LetterText::new(18, "R", Consonant::R),
            LetterText::new(19, "Z", Consonant::Z),
            LetterText::new(20, "C", Consonant::C),
            LetterText::new(21, "Q", Consonant::Q),
            LetterText::new(22, "G", Consonant::G),
            LetterText::new(23, "N", Consonant::N),
            LetterText::new(24, "V", Consonant::V),
            LetterText::new(25, "QU", Digraph::QU),
            LetterText::new(26, "H", Consonant::H),
            LetterText::new(27, "P", Consonant::P),
            LetterText::new(28, "W", Consonant::W),
            LetterText::new(29, "X", Consonant::X),
            LetterText::new(30, "F", Consonant::F),
            LetterText::new(31, "M", Consonant::M),
            LetterText::new(32, "S", Consonant::S),
            LetterText::new(33, "NG", Digraph::NG),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn should_split_mixed_case_digraph_letters() {
        let result: Vec<LetterText> =
            split_word_to_letters("tHThpHPhwHWhgHGhcHChsHShqUQunGNg").collect();

        let expected = [
            LetterText::new(0, "tH", Digraph::TH),
            LetterText::new(1, "Th", Digraph::TH),
            LetterText::new(2, "pH", Digraph::PH),
            LetterText::new(3, "Ph", Digraph::PH),
            LetterText::new(4, "wH", Digraph::WH),
            LetterText::new(5, "Wh", Digraph::WH),
            LetterText::new(6, "gH", Digraph::GH),
            LetterText::new(7, "Gh", Digraph::GH),
            LetterText::new(8, "cH", Digraph::CH),
            LetterText::new(9, "Ch", Digraph::CH),
            LetterText::new(10, "sH", Digraph::SH),
            LetterText::new(11, "Sh", Digraph::SH),
            LetterText::new(12, "qU", Digraph::QU),
            LetterText::new(13, "Qu", Digraph::QU),
            LetterText::new(14, "nG", Digraph::NG),
            LetterText::new(15, "Ng", Digraph::NG),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn should_filter_invalid_letters() {
        let result: Vec<LetterText> = split_word_to_letters(
            "äöüy̆+*~#'私i#あ-_.:,;<>|@n€^°1!2²\"3§³4$5v%6한글&7/{a8([9)l]0=i}ßd?\\´`",
        )
            .collect();

        let expected = [
            LetterText::new(0, "i", Vocal::I),
            LetterText::new(1, "n", Consonant::N),
            LetterText::new(2, "v", Consonant::V),
            LetterText::new(3, "a", Vocal::A),
            LetterText::new(4, "l", Consonant::L),
            LetterText::new(5, "i", Vocal::I),
            LetterText::new(6, "d", Consonant::D),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn should_convert_sentence() {
        let result = convert_sentence("mE ay̆nd yOöÜu");

        let me = WordText {
            local_index: 0,
            text: "mE".to_string(),
            letters: vec![
                LetterText::new(0, "m", Consonant::M),
                LetterText::new(1, "E", Vocal::E),
            ],
        };

        let and = WordText {
            local_index: 1,
            text: "and".to_string(),
            letters: vec![
                LetterText::new(0, "a", Vocal::A),
                LetterText::new(1, "n", Consonant::N),
                LetterText::new(2, "d", Consonant::D),
            ],
        };

        let you = WordText {
            local_index: 2,
            text: "yOu".to_string(),
            letters: vec![
                LetterText::new(0, "y", Consonant::Y),
                LetterText::new(1, "O", Vocal::O),
                LetterText::new(2, "u", Vocal::U),
            ],
        };

        let expected = SentenceText {
            local_index: 0,
            text: "mE and yOu".to_string(),
            words: vec![me, and, you],
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn should_send_create_events() {
        let mut app = App::new();
        app.add_plugins(TextConverterPlugin);

        app.world_mut().send_event(SetText("me yöou".to_string()));
        app.update();

        let events = app
            .world()
            .get_resource::<Events<TextModification>>()
            .unwrap();

        let mut reader = ManualEventReader::<TextModification>::default();

        let result: Vec<_> = reader.read(&events).cloned().collect();
        let expected = vec![
            TextModification::create((0, 0, 0), "m"),
            TextModification::create((0, 0, 1), "e"),
            TextModification::create((0, 1, 0), "y"),
            TextModification::create((0, 1, 1), "o"),
            TextModification::create((0, 1, 2), "u"),
        ];

        assert_eq!(result, expected);
    }

    fn change_text(from: &str, to: &str) -> Vec<TextModification> {
        let mut app = App::new();
        app.add_plugins(TextConverterPlugin);

        app.world_mut().send_event(SetText(from.to_string()));
        app.update();

        let events = app
            .world()
            .get_resource::<Events<TextModification>>()
            .unwrap();

        let mut reader = ManualEventReader::<TextModification>::default();
        reader.clear(&events);

        app.world_mut().send_event(SetText(to.to_string()));
        app.update();

        let events = app
            .world()
            .get_resource::<Events<TextModification>>()
            .unwrap();

        reader.read(&events).cloned().collect()
    }

    #[test]
    fn should_send_events_on_replace_single_consonant() {
        let result = change_text("me yöou", "me yöau");
        let expected = vec![
            TextModification::delete((0, 1, 1), "o"),
            TextModification::create((0, 1, 1), "a"),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn should_send_events_on_replace_with_digraph() {
        let result = change_text("me uq", "me ququ");
        let expected = vec![
            TextModification::delete((0, 1, 0), "u"),
            TextModification::delete((0, 1, 1), "q"),
            TextModification::create((0, 1, 0), "qu"),
            TextModification::create((0, 1, 1), "qu"),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn should_send_move_events() {
        let result = change_text("meand you", "me andyou");
        let expected = vec![
            TextModification::r#move((0, 0, 2), (0, 1, 0), "a"),
            TextModification::r#move((0, 0, 3), (0, 1, 1), "n"),
            TextModification::r#move((0, 0, 4), (0, 1, 2), "d"),
            TextModification::r#move((0, 1, 0), (0, 1, 3), "y"),
            TextModification::r#move((0, 1, 1), (0, 1, 4), "o"),
            TextModification::r#move((0, 1, 2), (0, 1, 5), "u"),
        ];

        assert_eq!(result, expected);
    }
}
