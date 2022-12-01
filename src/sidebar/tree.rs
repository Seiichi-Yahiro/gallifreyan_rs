use crate::image_types::{CircleChildren, Letter, LineSlotChildren, Sentence, Text, Word};
use crate::ui::tree::{CollapsingTreeItem, TreeItem};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::egui;

type WorldQuery = (
    Entity,
    &'static Text,
    &'static CircleChildren,
    &'static LineSlotChildren,
    &'static mut Openness,
);

type SentenceQuery<'w, 's> =
    Query<'w, 's, WorldQuery, (With<Sentence>, Without<Word>, Without<Letter>)>;
type WordQuery<'w, 's> =
    Query<'w, 's, WorldQuery, (With<Word>, Without<Sentence>, Without<Letter>)>;
type LetterQuery<'w, 's> =
    Query<'w, 's, WorldQuery, (With<Letter>, Without<Sentence>, Without<Word>)>;

#[derive(SystemParam)]
pub struct TreeSystemParams<'w, 's> {
    sentence_query: SentenceQuery<'w, 's>,
    word_query: WordQuery<'w, 's>,
    letter_query: LetterQuery<'w, 's>,
}

pub fn ui_tree(ui: &mut egui::Ui, mut params: TreeSystemParams) {
    for (sentence_entity, sentence_text, words, sentence_line_slots, mut openness) in
        params.sentence_query.iter_mut()
    {
        CollapsingTreeItem::new(sentence_text, sentence_entity, &mut openness).show(ui, |ui| {
            ui_words(ui, words, &mut params.word_query, &mut params.letter_query);
            ui_line_slots(ui, sentence_line_slots);
        });
    }
}

fn ui_words(
    ui: &mut egui::Ui,
    words: &[Entity],
    word_query: &mut WordQuery,
    letter_query: &mut LetterQuery,
) {
    let mut iter = word_query.iter_many_mut(words.iter());

    while let Some((word_entity, word_text, letters, word_line_slots, mut openness)) =
        iter.fetch_next()
    {
        CollapsingTreeItem::new(word_text, word_entity, &mut openness).show(ui, |ui| {
            ui_letters(ui, letters, letter_query);
            ui_line_slots(ui, word_line_slots);
        });
    }
}

fn ui_letters(ui: &mut egui::Ui, letters: &[Entity], letter_query: &mut LetterQuery) {
    let mut iter = letter_query.iter_many_mut(letters.iter());

    while let Some((letter_entity, letter_text, dots, letter_line_slots, mut openness)) =
        iter.fetch_next()
    {
        if dots.len() + letter_line_slots.len() == 0 {
            let tree_item = TreeItem::new(letter_text);
            ui.add(tree_item);
        } else {
            CollapsingTreeItem::new(letter_text, letter_entity, &mut openness).show(ui, |ui| {
                ui_dots(ui, dots);
                ui_line_slots(ui, letter_line_slots);
            });
        }
    }
}

fn ui_dots(ui: &mut egui::Ui, dots: &[Entity]) {
    for _dot_entity in dots.iter() {
        ui.add(TreeItem::new("Dot"));
    }
}

fn ui_line_slots(ui: &mut egui::Ui, line_slots: &[Entity]) {
    for _line_slot_entity in line_slots.iter() {
        ui.add(TreeItem::new("LINE"));
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Openness(bool);

pub fn add_openness(mut commands: Commands, query: Query<Entity, Added<CircleChildren>>) {
    for entity in query.iter() {
        commands.entity(entity).insert(Openness(false));
    }
}
