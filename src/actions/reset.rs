use crate::event_set::*;
use crate::image_types::{
    CircleChildren, Consonant, ConsonantPlacement, Dot, Letter, PositionData, Radius, Sentence,
    Vocal, VocalPlacement, Word,
};
use bevy::prelude::*;
use itertools::Itertools;

pub struct ResetPlugin;

impl Plugin for ResetPlugin {
    fn build(&self, app: &mut App) {
        app.add_event_set::<ResetActions>()
            .add_system(reset_circle_and_line_slots)
            .add_system(reset_sentences.after(reset_circle_and_line_slots))
            .add_system(reset_words.after(reset_sentences))
            .add_system(reset_letters.after(reset_words))
            .add_system(reset_vocals.after(reset_letters)) // TODO put after consonants? nested vocals
            .add_system(reset_consonants.after(reset_letters))
            .add_system(reset_dots.after(reset_consonants));
    }
}

pub struct ResetAll;
pub struct ResetSentence(pub Entity);
pub struct ResetWord(pub Entity);
pub struct ResetLetter(pub Entity);
pub struct ResetVocal(pub Entity);
pub struct ResetConsonant(pub Entity);
pub struct ResetDot(pub Entity);
pub struct ResetLineSlot(pub Entity);

event_set!(ResetActions {
    ResetSentence,
    ResetWord,
    ResetLetter,
    ResetVocal,
    ResetConsonant,
    ResetDot,
    ResetLineSlot
});

fn reset_circle_and_line_slots(
    sentence_query: Query<Entity, With<Sentence>>,
    mut events: EventReader<ResetAll>,
    mut reset_actions: ResetActions,
) {
    if events.iter().last().is_some() {
        for sentence in sentence_query.iter() {
            reset_actions.dispatch(ResetSentence(sentence));
        }
    }
}

fn reset_sentences(
    mut sentence_query: Query<(&mut PositionData, &mut Radius, &CircleChildren), With<Sentence>>,
    mut events: EventReader<ResetSentence>,
    mut reset_word: EventWriter<ResetWord>,
) {
    for ResetSentence(sentence) in events.iter() {
        if let Ok((mut position_data, mut radius, words)) = sentence_query.get_mut(*sentence) {
            position_data.angle = 0.0;
            position_data.distance = 0.0;

            **radius = (1000.0 / 2.0) * 0.9;

            for word in words.iter() {
                reset_word.send(ResetWord(*word));
            }
        } else {
            error!("Couldn't find sentence entity to reset!");
        }
    }
}

fn reset_words(
    mut word_query: Query<(&mut PositionData, &mut Radius, &CircleChildren, &Parent), With<Word>>,
    parent_query: Query<(&Radius, &CircleChildren), (With<Sentence>, Without<Word>)>,
    mut events: EventReader<ResetWord>,
    mut reset_letters: EventWriter<ResetLetter>,
) {
    for ResetWord(word) in events.iter() {
        if let Ok((mut position_data, mut radius, letters, parent)) = word_query.get_mut(*word) {
            if let Ok((Radius(sentence_radius), words)) = parent_query.get(parent.get()) {
                **radius = (sentence_radius * 0.75) / (1.0 + words.len() as f32 / 2.0);

                position_data.distance = if words.len() > 1 {
                    sentence_radius - **radius * 1.5
                } else {
                    0.0
                };

                let word_index = words
                    .iter()
                    .find_position(|entity| **entity == *word)
                    .map(|(index, _)| index);

                if let Some(word_index) = word_index {
                    position_data.angle = word_index as f32 * (360.0 / words.len() as f32);
                } else {
                    error!("Couldn't find word index to reset position!");
                }

                for letter in letters.iter() {
                    reset_letters.send(ResetLetter(*letter));
                }
            } else {
                error!("Couldn't find parent sentence entity while resetting word!");
            }
        } else {
            error!("Couldn't find word entity to reset!");
        }
    }
}

fn reset_letters(
    letter_query: Query<(Option<&Vocal>, Option<&Consonant>), With<Letter>>,
    mut events: EventReader<ResetLetter>,
    mut reset_vocals: EventWriter<ResetVocal>,
    mut reset_consonants: EventWriter<ResetConsonant>,
) {
    for ResetLetter(letter) in events.iter() {
        if let Ok((vocal, consonant)) = letter_query.get(*letter) {
            if vocal.is_some() {
                reset_vocals.send(ResetVocal(*letter));
            } else if consonant.is_some() {
                reset_consonants.send(ResetConsonant(*letter));
            }
        } else {
            error!("Couldn't find letter entity to reset!");
        }
    }
}

fn reset_vocals(
    mut vocal_query: Query<(&mut PositionData, &mut Radius, &VocalPlacement, &Parent), With<Vocal>>,
    parent_query: Query<(&Radius, &CircleChildren), (With<Word>, Without<Vocal>)>,
    mut events: EventReader<ResetVocal>,
) {
    for ResetVocal(vocal) in events.iter() {
        if let Ok((mut position_data, mut radius, placement, parent)) = vocal_query.get_mut(*vocal)
        {
            if let Ok((Radius(word_radius), letters)) = parent_query.get(parent.get()) {
                **radius = (word_radius * 0.75 * 0.4) / (1.0 + letters.len() as f32 / 2.0);

                position_data.distance = match placement {
                    VocalPlacement::OnLine => *word_radius,
                    VocalPlacement::Outside => word_radius + **radius * 1.5,
                    VocalPlacement::Inside => {
                        if letters.len() > 1 {
                            word_radius - **radius * 1.5
                        } else {
                            0.0
                        }
                    }
                };

                let letter_index = letters
                    .iter()
                    .find_position(|entity| **entity == *vocal)
                    .map(|(index, _)| index);

                if let Some(letter_index) = letter_index {
                    position_data.angle = letter_index as f32 * (360.0 / letters.len() as f32);
                } else {
                    error!("Couldn't find vocal index to reset position!");
                }
            } else {
                error!("Couldn't find parent word entity while resetting vocal!");
            }
        } else {
            error!("Couldn't find vocal entity to reset!");
        }
    }
}

fn reset_consonants(
    mut consonant_query: Query<
        (
            &mut PositionData,
            &mut Radius,
            &ConsonantPlacement,
            &CircleChildren,
            &Parent,
        ),
        With<Consonant>,
    >,
    parent_query: Query<(&Radius, &CircleChildren), (With<Word>, Without<Consonant>)>,
    mut events: EventReader<ResetConsonant>,
    mut reset_dots: EventWriter<ResetDot>,
) {
    for ResetConsonant(consonant) in events.iter() {
        if let Ok((mut position_data, mut radius, placement, dots, parent)) =
            consonant_query.get_mut(*consonant)
        {
            if let Ok((Radius(word_radius), letters)) = parent_query.get(parent.get()) {
                **radius = (word_radius * 0.75) / (1.0 + letters.len() as f32 / 2.0);

                position_data.distance = match placement {
                    ConsonantPlacement::DeepCut => word_radius - **radius * 0.75,
                    ConsonantPlacement::Inside => {
                        if letters.len() > 1 {
                            word_radius - **radius * 1.5
                        } else {
                            0.0
                        }
                    }
                    ConsonantPlacement::ShallowCut => *word_radius,
                    ConsonantPlacement::OnLine => *word_radius,
                };

                let letter_index = letters
                    .iter()
                    .find_position(|entity| **entity == *consonant)
                    .map(|(index, _)| index);

                if let Some(letter_index) = letter_index {
                    position_data.angle = letter_index as f32 * (360.0 / letters.len() as f32);
                } else {
                    error!("Couldn't find vocal index to reset position!");
                }

                for dot in dots.iter() {
                    reset_dots.send(ResetDot(*dot));
                }
            } else {
                error!("Couldn't find parent word entity while resetting consonant!");
            }
        } else {
            error!("Couldn't find consonant entity to reset!");
        }
    }
}

fn reset_dots(
    mut dot_query: Query<(&mut PositionData, &mut Radius, &Parent), With<Dot>>,
    parent_query: Query<(&Radius, &CircleChildren), (With<Consonant>, Without<Dot>)>,
    mut events: EventReader<ResetDot>,
) {
    for ResetDot(dot) in events.iter() {
        if let Ok((mut position_data, mut radius, parent)) = dot_query.get_mut(*dot) {
            if let Ok((Radius(consonant_radius), dots)) = parent_query.get(parent.get()) {
                const LETTER_SIDE_ANGLE: f32 = 180.0;
                const DOT_DISTANCE_ANGLE: f32 = 45.0;

                let center_dots_on_letter_side_angle: f32 =
                    ((dots.len() - 1) as f32 * DOT_DISTANCE_ANGLE) / 2.0;

                **radius = consonant_radius * 0.1;

                position_data.distance = consonant_radius - **radius * 1.5;

                let dot_index = dots
                    .iter()
                    .find_position(|entity| **entity == *dot)
                    .map(|(index, _)| index);

                if let Some(dot_index) = dot_index {
                    position_data.angle = dot_index as f32 * DOT_DISTANCE_ANGLE
                        - center_dots_on_letter_side_angle
                        + LETTER_SIDE_ANGLE;
                } else {
                    error!("Couldn't find dot index to reset position!");
                }
            } else {
                error!("Couldn't find parent consonant entity while resetting dot!");
            }
        } else {
            error!("Couldn't find dot entity to reset!");
        }
    }
}
