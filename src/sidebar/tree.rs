use crate::events::{Select, Selection};
use crate::image_types::{CircleChildren, Letter, LineSlotChildren, Sentence, Text, Word};
use crate::ui::tree::CollapsingTreeItem;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::egui;
use itertools::Itertools;

type WorldQuery = (
    Entity,
    &'static Text,
    &'static CircleChildren,
    &'static LineSlotChildren,
    &'static mut IsOpen,
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
    select_event: EventWriter<'w, 's, Select>,
    selection: Res<'w, Selection>,
}

pub fn ui_tree(ui: &mut egui::Ui, mut params: TreeSystemParams) {
    egui::CentralPanel::default().show_inside(ui, |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for (sentence_entity, sentence_text, words, sentence_line_slots, mut is_open) in
                    params.sentence_query.iter_mut()
                {
                    let (header_response, _) = CollapsingTreeItem::new(
                        sentence_text,
                        sentence_entity,
                        &mut is_open,
                        params.selection.iter().contains(&sentence_entity),
                    )
                    .show(ui, |ui| {
                        ui_words(
                            ui,
                            words,
                            &mut params.word_query,
                            &mut params.letter_query,
                            &mut params.select_event,
                            &params.selection,
                        );
                        ui_line_slots(
                            ui,
                            sentence_line_slots,
                            &mut params.select_event,
                            &params.selection,
                        );
                    });

                    if header_response.clicked() {
                        params.select_event.send(Select(Some(sentence_entity)));
                    }
                }
            });

        let free_space_response = ui.allocate_response(ui.available_size(), egui::Sense::click());
        if free_space_response.clicked() {
            params.select_event.send(Select(None));
        }
    });
}

fn ui_words(
    ui: &mut egui::Ui,
    words: &[Entity],
    word_query: &mut WordQuery,
    letter_query: &mut LetterQuery,
    select_event: &mut EventWriter<Select>,
    selection: &Res<Selection>,
) {
    let mut iter = word_query.iter_many_mut(words.iter());

    while let Some((word_entity, word_text, letters, word_line_slots, mut is_open)) =
        iter.fetch_next()
    {
        let (header_response, _) = CollapsingTreeItem::new(
            word_text,
            word_entity,
            &mut is_open,
            selection.iter().contains(&word_entity),
        )
        .show(ui, |ui| {
            ui_letters(ui, letters, letter_query, select_event, selection);
            ui_line_slots(ui, word_line_slots, select_event, selection);
        });

        if header_response.clicked() {
            select_event.send(Select(Some(word_entity)));
        }
    }
}

fn ui_letters(
    ui: &mut egui::Ui,
    letters: &[Entity],
    letter_query: &mut LetterQuery,
    select_event: &mut EventWriter<Select>,
    selection: &Res<Selection>,
) {
    let mut iter = letter_query.iter_many_mut(letters.iter());

    while let Some((letter_entity, letter_text, dots, letter_line_slots, mut is_open)) =
        iter.fetch_next()
    {
        let is_selected = selection.iter().contains(&letter_entity);

        let header_response = if dots.len() + letter_line_slots.len() == 0 {
            CollapsingTreeItem::new_empty(ui, letter_text, letter_entity, is_selected)
        } else {
            let (header_response, _) =
                CollapsingTreeItem::new(letter_text, letter_entity, &mut is_open, is_selected)
                    .show(ui, |ui| {
                        ui_dots(ui, dots, select_event, selection);
                        ui_line_slots(ui, letter_line_slots, select_event, selection);
                    });

            header_response
        };

        if header_response.clicked() {
            select_event.send(Select(Some(letter_entity)));
        }
    }
}

fn ui_dots(
    ui: &mut egui::Ui,
    dots: &[Entity],
    select_event: &mut EventWriter<Select>,
    selection: &Res<Selection>,
) {
    for dot_entity in dots.iter() {
        let header_response = CollapsingTreeItem::new_empty(
            ui,
            "DOT",
            dot_entity,
            selection.iter().contains(&dot_entity),
        );
        if header_response.clicked() {
            select_event.send(Select(Some(*dot_entity)));
        }
    }
}

fn ui_line_slots(
    ui: &mut egui::Ui,
    line_slots: &[Entity],
    select_event: &mut EventWriter<Select>,
    selection: &Res<Selection>,
) {
    for line_slot_entity in line_slots.iter() {
        let header_response = CollapsingTreeItem::new_empty(
            ui,
            "LINE",
            line_slot_entity,
            selection.iter().contains(&line_slot_entity),
        );
        if header_response.clicked() {
            select_event.send(Select(Some(*line_slot_entity)));
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct IsOpen(bool);

pub fn add_is_open_component(mut commands: Commands, query: Query<Entity, Added<CircleChildren>>) {
    for entity in query.iter() {
        commands.entity(entity).insert(IsOpen(false));
    }
}
