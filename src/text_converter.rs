use crate::image_types::*;
use crate::ui::TreeNode;
use bevy::prelude::*;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

lazy_static! {
    static ref VALID_LETTER: Regex = RegexBuilder::new(r"[cpwstg]h?|ng?|qu?|[aeioubdhfjklmrvyzx]")
        .case_insensitive(true)
        .build()
        .unwrap();
    static ref VOCAL: Regex = RegexBuilder::new(r"^[aeiou]$")
        .case_insensitive(true)
        .build()
        .unwrap();
}

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

pub fn spawn_sentence(commands: &mut Commands, text: &str) -> TreeNode {
    let mut word_nodes: Vec<TreeNode> = Vec::with_capacity(10);

    for word in text.split_whitespace() {
        let node = spawn_word(commands, word);
        word_nodes.push(node);
    }

    let word_entities: Vec<Entity> = word_nodes.iter().map(|node| node.entity).collect();

    let mut sentence = commands.spawn_empty();
    sentence.push_children(&word_entities);

    sentence.insert(SentenceBundle::new(
        Text(text.to_string()),
        CircleChildren(word_entities),
        LineSlotChildren(vec![]),
    ));

    TreeNode {
        entity: sentence.id(),
        text: text.to_string(),
        open: true,
        children: word_nodes,
    }
}

fn spawn_word(commands: &mut Commands, text: &str) -> TreeNode {
    let letters = split_word_to_chars(text);
    let mut letter_nodes: Vec<TreeNode> = Vec::with_capacity(5);

    for letter in letters {
        let node = if VOCAL.is_match(letter) {
            spawn_vocal(commands, letter)
        } else {
            spawn_consonant(commands, letter)
        };

        letter_nodes.push(node);
    }

    let letter_entities: Vec<Entity> = letter_nodes.iter().map(|node| node.entity).collect();

    let mut word = commands.spawn_empty();
    word.push_children(&letter_entities);

    word.insert(WordBundle::new(
        Text(text.to_string()),
        CircleChildren(letter_entities),
        LineSlotChildren(vec![]),
    ));

    TreeNode {
        entity: word.id(),
        text: text.to_string(),
        open: false,
        children: letter_nodes,
    }
}

fn spawn_vocal(commands: &mut Commands, text: &str) -> TreeNode {
    let decoration = VocalDecoration::try_from(text).unwrap();
    let placement = VocalPlacement::try_from(text).unwrap();

    let line_slot_node = spawn_vocal_line(commands, decoration);
    let line_slot_entity = line_slot_node.as_ref().map(|node| node.entity);

    let mut vocal = commands.spawn_empty();
    if let Some(line_slot) = line_slot_entity {
        vocal.add_child(line_slot);
    }

    vocal.insert(VocalBundle::new(
        Text(text.to_string()),
        placement,
        decoration,
        LineSlotChildren(line_slot_entity.into_iter().collect()),
    ));

    TreeNode {
        entity: vocal.id(),
        text: text.to_string(),
        open: false,
        children: line_slot_node.into_iter().collect(),
    }
}

fn spawn_vocal_line(commands: &mut Commands, decoration: VocalDecoration) -> Option<TreeNode> {
    match decoration {
        VocalDecoration::None => None,
        VocalDecoration::LineInside | VocalDecoration::LineOutside => {
            let line_slot = commands
                .spawn(LineSlotBundle {
                    line_slot: LineSlot,
                    position_data: Default::default(),
                })
                .id();

            let node = TreeNode {
                entity: line_slot,
                text: "LINE".to_string(),
                open: false,
                children: vec![],
            };

            Some(node)
        }
    }
}

fn spawn_consonant(commands: &mut Commands, text: &str) -> TreeNode {
    let decoration = ConsonantDecoration::try_from(text).unwrap();
    let placement = ConsonantPlacement::try_from(text).unwrap();

    let dot_nodes = spawn_dots(commands, decoration);
    let dot_entities: Vec<Entity> = dot_nodes.iter().map(|node| node.entity).collect();

    let line_slot_nodes = spawn_consonant_lines(commands, decoration);
    let line_slot_entities: Vec<Entity> = line_slot_nodes.iter().map(|node| node.entity).collect();

    let mut consonant = commands.spawn_empty();
    consonant.push_children(&dot_entities);
    consonant.push_children(&line_slot_entities);

    consonant.insert(ConsonantBundle::new(
        Text(text.to_string()),
        placement,
        decoration,
        CircleChildren(dot_entities),
        LineSlotChildren(line_slot_entities),
    ));

    TreeNode {
        entity: consonant.id(),
        text: text.to_string(),
        open: false,
        children: dot_nodes
            .into_iter()
            .chain(line_slot_nodes.into_iter())
            .collect(),
    }
}

fn spawn_dots(commands: &mut Commands, decoration: ConsonantDecoration) -> Vec<TreeNode> {
    let number_of_dots = match decoration {
        ConsonantDecoration::SingleDot => 1,
        ConsonantDecoration::DoubleDot => 2,
        ConsonantDecoration::TripleDot => 3,
        ConsonantDecoration::QuadrupleDot => 4,
        _ => 0,
    };

    let mut dots: Vec<TreeNode> = Vec::with_capacity(number_of_dots);

    for _ in 0..number_of_dots {
        let dot = commands.spawn(DotBundle::new()).id();

        let node = TreeNode {
            entity: dot,
            text: "DOT".to_string(),
            open: false,
            children: vec![],
        };

        dots.push(node);
    }

    dots
}

fn spawn_consonant_lines(
    commands: &mut Commands,
    decoration: ConsonantDecoration,
) -> Vec<TreeNode> {
    let number_of_lines = match decoration {
        ConsonantDecoration::SingleLine => 1,
        ConsonantDecoration::DoubleLine => 2,
        ConsonantDecoration::TripleLine => 3,
        _ => 0,
    };

    let mut line_slots: Vec<TreeNode> = Vec::with_capacity(number_of_lines);

    for _ in 0..number_of_lines {
        let line_slot = commands
            .spawn(LineSlotBundle {
                line_slot: LineSlot,
                position_data: Default::default(),
            })
            .id();

        let node = TreeNode {
            entity: line_slot,
            text: "LINE".to_string(),
            open: false,
            children: vec![],
        };

        line_slots.push(node);
    }

    line_slots
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
