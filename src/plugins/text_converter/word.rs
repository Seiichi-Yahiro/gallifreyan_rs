use crate::math::Degree;
use crate::plugins::text_converter::prelude::*;
use bevy::prelude::*;

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct Word;

#[derive(Bundle)]
pub struct WordBundle {
    pub name: Name,
    pub word: Word,
    pub text: GFText,
    pub letters: CircleChildren,
    pub line_slots: LineSlotChildren,
    pub sibling_index: SiblingIndex,
    pub radius: Radius,
    pub position_data: PositionData,
}

impl WordBundle {
    pub fn new(word: String, sibling_index: usize) -> Self {
        Self {
            name: Name::new("Word"),
            word: Word,
            text: GFText(word),
            letters: CircleChildren::default(),
            line_slots: LineSlotChildren::default(),
            sibling_index: SiblingIndex(sibling_index),
            radius: Radius::default(),
            position_data: PositionData::default(),
        }
    }
}

pub fn convert_words(
    mut commands: Commands,
    mut sentence_query: Query<
        (Entity, &GFText, &mut CircleChildren),
        (With<Sentence>, Changed<GFText>),
    >,
    mut word_query: Query<
        (Entity, &mut GFText, &mut SiblingIndex),
        (With<Word>, Without<Sentence>),
    >,
) {
    for (sentence_entity, sentence_text, mut children) in sentence_query.iter_mut() {
        let mut existing_words = word_query.iter_many_mut(children.0.iter());
        let mut new_words_iter = sentence_text.0.split_whitespace().map(String::from);

        let mut new_children: Vec<Entity> = Vec::new();

        loop {
            let next_exiting_word = existing_words.fetch_next();
            let next_new_word = new_words_iter.next();
            let sibling_index = new_children.len();

            match (next_exiting_word, next_new_word) {
                // update word
                (Some((word_entity, mut word_text, mut word_sibling_index)), Some(new_word)) => {
                    if word_text.0 != new_word {
                        debug!(
                            "Update word: {} -> {}, {:?}",
                            word_text.0, new_word, word_entity
                        );
                        word_text.0 = new_word;
                    }

                    word_sibling_index.0 = sibling_index;
                    new_children.push(word_entity);
                }
                // remove word
                (Some((word_entity, word_text, _word_sibling_index)), None) => {
                    debug!("Despawn word: {}, {:?}", word_text.0, word_entity);
                    commands.entity(word_entity).despawn_recursive();
                }
                // add word
                (None, Some(new_word)) => {
                    let bundle = WordBundle::new(new_word.clone(), sibling_index);

                    let word_entity = commands.spawn(bundle).id();
                    debug!("Spawn word: {}, {:?}", new_word, word_entity);

                    commands.entity(sentence_entity).add_child(word_entity);
                    new_children.push(word_entity);
                }
                (None, None) => {
                    break;
                }
            }
        }

        children.0 = new_children;
    }
}

pub fn set_default_radius(
    sentence_query: Query<(&Radius, &CircleChildren), (With<Sentence>, Without<Word>)>,
    mut word_query: Query<(&Parent, &mut Radius), (With<Word>, Without<Sentence>)>,
) {
    for (word_parent, mut word_radius) in word_query.iter_mut() {
        let (sentence_radius, sentence_children) = sentence_query.get(word_parent.get()).unwrap();

        let number_of_words = sentence_children.0.len() as f32;

        word_radius.0 = (sentence_radius.0 * 0.75) / (1.0 + number_of_words / 2.0);
    }
}

pub fn set_default_position(
    sentence_query: Query<(&Radius, &CircleChildren), (With<Sentence>, Without<Word>)>,
    mut word_query: Query<
        (&Parent, &mut PositionData, &Radius, &SiblingIndex),
        (With<Word>, Without<Sentence>),
    >,
) {
    for (word_parent, mut position_data, word_radius, word_index) in word_query.iter_mut() {
        let (sentence_radius, sentence_children) = sentence_query.get(word_parent.get()).unwrap();

        let number_of_words = sentence_children.0.len();

        if number_of_words > 1 {
            position_data.distance = sentence_radius.0 - word_radius.0 * 1.5;
        } else {
            position_data.distance = 0.0;
        }

        position_data.angle = Degree(word_index.0 as f32 * (360.0 / number_of_words as f32));
    }
}
