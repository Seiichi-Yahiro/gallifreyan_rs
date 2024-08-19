use crate::plugins::text_converter::letter::consonant::{Consonant, ConsonantCluster};
use crate::plugins::text_converter::letter::vocal::Vocal;
use crate::plugins::text_converter::letter::Letter;
use crate::plugins::text_converter::LetterText;

pub trait CombineLetters: Iterator<Item = LetterText> {
    fn combine(self) -> LetterCombinator<Self>
    where
        Self: Sized,
    {
        LetterCombinator {
            iterator: self,
            prev: None,
            combined_count: 0,
        }
    }
}

impl<I> CombineLetters for I where I: Iterator<Item = LetterText> {}

pub struct LetterCombinator<I>
where
    I: Iterator<Item = LetterText>,
{
    iterator: I,
    prev: Option<LetterText>,
    combined_count: usize,
}

impl<I> Iterator for LetterCombinator<I>
where
    I: Iterator<Item = LetterText>,
{
    type Item = LetterText;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match (self.prev.take(), self.iterator.next()) {
                (None, Some(nex_letter_text)) => {
                    self.prev = Some(nex_letter_text);
                }
                (
                    Some(LetterText {
                        local_index,
                        text: prev_grapheme,
                        letter:
                            Letter::Consonant(ConsonantCluster::Single(
                                Consonant::T
                                | Consonant::P
                                | Consonant::W
                                | Consonant::G
                                | Consonant::C
                                | Consonant::S,
                            )),
                    }),
                    Some(LetterText {
                        text: next_grapheme,
                        letter: Letter::Consonant(ConsonantCluster::Single(Consonant::H)),
                        ..
                    }),
                )
                | (
                    Some(LetterText {
                        local_index,
                        text: prev_grapheme,
                        letter: Letter::Consonant(ConsonantCluster::Single(Consonant::N)),
                    }),
                    Some(LetterText {
                        text: next_grapheme,
                        letter: Letter::Consonant(ConsonantCluster::Single(Consonant::G)),
                        ..
                    }),
                )
                | (
                    Some(LetterText {
                        local_index,
                        text: prev_grapheme,
                        letter: Letter::Consonant(ConsonantCluster::Single(Consonant::Q)),
                    }),
                    Some(LetterText {
                        text: next_grapheme,
                        letter: Letter::Vocal(Vocal::U),
                        ..
                    }),
                ) => {
                    let grapheme = prev_grapheme + &next_grapheme;
                    let letter = Letter::try_from(grapheme.as_str()).unwrap();

                    let letter_text = LetterText {
                        local_index: local_index - self.combined_count,
                        text: grapheme,
                        letter,
                    };

                    self.combined_count += 1;

                    return Some(letter_text);
                }
                (Some(prev), Some(next)) => {
                    self.prev = Some(next);
                    return Some(LetterText {
                        local_index: prev.local_index - self.combined_count,
                        ..prev
                    });
                }
                (Some(prev), None) => {
                    return Some(LetterText {
                        local_index: prev.local_index - self.combined_count,
                        ..prev
                    });
                }
                (None, None) => {
                    return None;
                }
            }
        }
    }
}
