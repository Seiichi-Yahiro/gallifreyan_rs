use crate::plugins::selection::{Select, Selected};
use crate::plugins::text_converter::components::{
    CircleChildren, Letter, LineSlotChildren, NestedLetter, NestedVocal, Sentence, Text, Word,
};
use crate::plugins::ui::widgets::tree::CollapsingTreeItem;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::egui;

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

type LetterQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Text,
        &'static CircleChildren,
        &'static LineSlotChildren,
        &'static NestedLetter,
        &'static mut IsOpen,
    ),
    (
        With<Letter>,
        Without<Sentence>,
        Without<Word>,
        Without<NestedVocal>,
    ),
>;

type NestedLetterQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Text,
        &'static LineSlotChildren,
        &'static mut IsOpen,
    ),
    (
        With<Letter>,
        With<NestedVocal>,
        Without<Sentence>,
        Without<Word>,
    ),
>;

#[derive(SystemParam)]
pub struct TreeSystemParams<'w, 's> {
    sentence_query: SentenceQuery<'w, 's>,
    word_query: WordQuery<'w, 's>,
    letter_query: LetterQuery<'w, 's>,
    nested_letter_query: NestedLetterQuery<'w, 's>,
    select_event: EventWriter<'w, Select>,
    selected_query: Query<'w, 's, Entity, With<Selected>>,
}

pub fn ui_tree(ui: &mut egui::Ui, mut params: TreeSystemParams) {
    let selection = params.selected_query.get_single().ok();

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
                        selection.contains(&sentence_entity),
                    )
                    .show(ui, |ui| {
                        ui_words(
                            ui,
                            words,
                            &mut params.word_query,
                            &mut params.letter_query,
                            &mut params.nested_letter_query,
                            &mut params.select_event,
                            &selection,
                        );
                        ui_line_slots(
                            ui,
                            sentence_line_slots,
                            &mut params.select_event,
                            &selection,
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
    nested_letter_query: &mut NestedLetterQuery,
    select_event: &mut EventWriter<Select>,
    selection: &Option<Entity>,
) {
    let mut iter = word_query.iter_many_mut(words.iter());

    while let Some((word_entity, word_text, letters, word_line_slots, mut is_open)) =
        iter.fetch_next()
    {
        let (header_response, _) = CollapsingTreeItem::new(
            word_text,
            word_entity,
            &mut is_open,
            selection.contains(&word_entity),
        )
        .show(ui, |ui| {
            ui_letters(
                ui,
                letters,
                letter_query,
                nested_letter_query,
                select_event,
                selection,
            );
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
    nested_letter_query: &mut NestedLetterQuery,
    select_event: &mut EventWriter<Select>,
    selection: &Option<Entity>,
) {
    let mut iter = letter_query.iter_many_mut(letters.iter());

    while let Some((letter_entity, letter_text, dots, letter_line_slots, nested, mut is_open)) =
        iter.fetch_next()
    {
        let is_selected = selection.contains(&letter_entity);

        let header_response = if dots.len() + letter_line_slots.len() + nested.iter().len() == 0 {
            CollapsingTreeItem::new_empty(ui, letter_text, letter_entity, is_selected)
        } else {
            let (header_response, _) =
                CollapsingTreeItem::new(letter_text, letter_entity, &mut is_open, is_selected)
                    .show(ui, |ui| {
                        ui_nested_letters(ui, nested, nested_letter_query, select_event, selection);
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

fn ui_nested_letters(
    ui: &mut egui::Ui,
    nested_letter: &Option<Entity>,
    nested_letter_query: &mut NestedLetterQuery,
    select_event: &mut EventWriter<Select>,
    selection: &Option<Entity>,
) {
    if let Some(nested_letter) = nested_letter {
        if let Ok((letter_entity, letter_text, letter_line_slots, mut is_open)) =
            nested_letter_query.get_mut(*nested_letter)
        {
            let is_selected = selection.contains(&letter_entity);

            let header_response = if letter_line_slots.is_empty() {
                CollapsingTreeItem::new_empty(ui, letter_text, letter_entity, is_selected)
            } else {
                let (header_response, _) =
                    CollapsingTreeItem::new(letter_text, letter_entity, &mut is_open, is_selected)
                        .show(ui, |ui| {
                            ui_line_slots(ui, letter_line_slots, select_event, selection);
                        });

                header_response
            };

            if header_response.clicked() {
                select_event.send(Select(Some(letter_entity)));
            }
        }
    }
}

fn ui_dots(
    ui: &mut egui::Ui,
    dots: &[Entity],
    select_event: &mut EventWriter<Select>,
    selection: &Option<Entity>,
) {
    for dot_entity in dots.iter() {
        let header_response =
            CollapsingTreeItem::new_empty(ui, "DOT", dot_entity, selection.contains(dot_entity));
        if header_response.clicked() {
            select_event.send(Select(Some(*dot_entity)));
        }
    }
}

fn ui_line_slots(
    ui: &mut egui::Ui,
    line_slots: &[Entity],
    select_event: &mut EventWriter<Select>,
    selection: &Option<Entity>,
) {
    for line_slot_entity in line_slots.iter() {
        let header_response = CollapsingTreeItem::new_empty(
            ui,
            "LINE",
            line_slot_entity,
            selection.contains(line_slot_entity),
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
