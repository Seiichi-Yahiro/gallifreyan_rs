use crate::plugins::text_converter::prelude::*;
use bevy::ecs::query::QuerySingleError;
use bevy::prelude::*;
use itertools::Itertools;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct Sentence;

#[derive(Bundle)]
pub struct SentenceBundle {
    pub name: Name,
    pub sentence: Sentence,
    pub text: GFText,
    pub words: CircleChildren,
    pub line_slots: LineSlotChildren,
    pub sibling_index: SiblingIndex,
}

impl SentenceBundle {
    pub fn new(sentence: String, sibling_index: usize) -> Self {
        Self {
            name: Name::new("Sentence"),
            sentence: Sentence,
            text: GFText(sentence),
            words: CircleChildren::default(),
            line_slots: LineSlotChildren::default(),
            sibling_index: SiblingIndex(sibling_index),
        }
    }
}

fn sanitize_sentence(sentence: &str) -> String {
    sentence
        .split_whitespace()
        .map(|word| {
            word.graphemes(true)
                .filter(|grapheme| Letter::try_from(*grapheme).is_ok())
                .join("")
        })
        .join(" ")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct SetSentenceSet;

#[derive(Debug, Event)]
pub struct SetSentence(pub String);

pub fn set_sentence(
    mut commands: Commands,
    mut events: EventReader<SetSentence>,
    mut sentence_query: Query<(Entity, &mut GFText), With<Sentence>>,
) {
    let Some(SetSentence(new_sentence)) = events.read().last() else {
        return;
    };

    let new_sentence = sanitize_sentence(new_sentence);

    match sentence_query.get_single_mut() {
        Ok((entity, mut text)) => {
            if new_sentence.is_empty() {
                debug!("Despawn sentence: {}, {:?}", text.0, entity);

                commands.entity(entity).despawn_recursive();
            } else if text.0 != new_sentence {
                debug!(
                    "Update sentence: {} -> {}, {:?}",
                    text.0, new_sentence, entity
                );

                text.0 = new_sentence;
            }
        }
        Err(QuerySingleError::NoEntities(_)) => {
            if new_sentence.is_empty() {
                return;
            }

            let bundle = SentenceBundle::new(new_sentence.clone(), 0);
            let entity = commands.spawn(bundle).id();
            debug!("Spawn sentence: {}, {:?}", new_sentence, entity);
        }
        Err(QuerySingleError::MultipleEntities(_)) => {
            error!("Only one sentence is currently supported");
        }
    }
}

#[cfg(test)]
mod test {
    use super::sanitize_sentence;

    #[test]
    fn should_sanitize() {
        let result = sanitize_sentence(
            "äöüy̆+*~#'私i#あ-_.:,;<>|@n€^°1!2²\"3§³4$5v%6한글&7/{a8([9)l]0=i}ßd?\\´`     text",
        );

        let expected = "invalid text";

        assert_eq!(result, expected);
    }
}
