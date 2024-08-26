mod components;
mod dot;
mod letter;
mod line_slot;
mod sentence;
mod word;

#[allow(unused_imports)]
pub mod prelude {
    pub use super::components::{CircleChildren, GFText, LineSlotChildren, SiblingIndex};
    pub use super::dot::{Dot, DotBundle};
    pub use super::letter::{
        combinator::{CombineLetters, LetterCombinator, LetterText},
        consonant::{
            Consonant, ConsonantCluster, ConsonantDecoration, ConsonantDecorationT,
            ConsonantPlacement, ConsonantPlacementT, Digraph,
        },
        vocal::{Vocal, VocalDecoration, VocalPlacement},
        Decorated, Letter, LetterBundle,
    };
    pub use super::line_slot::{LineSlot, LineSlotBundle};
    pub use super::sentence::{Sentence, SentenceBundle, SetSentence, SetSentenceSet};
    pub use super::word::{Word, WordBundle};
}

use crate::plugins::text_converter::dot::convert_dots;
use crate::plugins::text_converter::letter::convert_letters;
use crate::plugins::text_converter::line_slot::convert_line_slots;
use crate::plugins::text_converter::sentence::{set_sentence, SetSentence, SetSentenceSet};
use crate::plugins::text_converter::word::convert_words;
use bevy::prelude::*;

pub struct TextConverterPlugin;

impl Plugin for TextConverterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetSentence>()
            .configure_sets(
                Update,
                (
                    TextConversionSet.in_set(SetSentenceSet),
                    SetSentenceSet.run_if(on_event::<SetSentence>()),
                ),
            )
            .add_systems(
                Update,
                (
                    set_sentence,
                    convert_words,
                    convert_letters,
                    convert_dots,
                    convert_line_slots,
                )
                    .chain()
                    .in_set(TextConversionSet),
            );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct TextConversionSet;
