mod text_converter;

use crate::image_types::{CircleChildren, Letter, LineSlotChildren, Sentence, Text, Word};
use crate::sidebar::text_converter::{SetText, TextConverterPlugin};
use crate::ui::tree::{CollapsingTreeItem, TreeItem};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

pub struct SideBarPlugin;

impl Plugin for SideBarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SidebarState>()
            .add_system(ui.label(UiSystemLabel))
            .add_system(add_openness)
            .add_plugin(TextConverterPlugin);
    }
}

#[derive(Resource, Default)]
pub struct SidebarState {
    pub text: String,
    pub sanitized_text: String,
}

#[derive(SystemLabel)]
pub struct UiSystemLabel;

fn ui(
    mut egui_context: ResMut<EguiContext>,
    text_input_system_params: TextInputSystemParams,
    tree_system_params: TreeSystemParams,
) {
    egui::SidePanel::left("sidebar")
        .resizable(true)
        .show(egui_context.ctx_mut(), |ui| {
            ui_text_input(ui, text_input_system_params);

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui_tree(ui, tree_system_params);
            });
        });
}

#[derive(SystemParam)]
struct TextInputSystemParams<'w, 's> {
    ui_state: ResMut<'w, SidebarState>,
    set_text_event: EventWriter<'w, 's, SetText>,
}

fn ui_text_input(ui: &mut egui::Ui, mut params: TextInputSystemParams) {
    if ui.text_edit_singleline(&mut params.ui_state.text).changed() {
        let new_sanitized_text = text_converter::sanitize_text_input(&params.ui_state.text);
        if params.ui_state.sanitized_text != new_sanitized_text {
            params.ui_state.sanitized_text = new_sanitized_text;
            params
                .set_text_event
                .send(SetText(params.ui_state.sanitized_text.clone()));
        }
    }
}

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
struct TreeSystemParams<'w, 's> {
    sentence_query: SentenceQuery<'w, 's>,
    word_query: WordQuery<'w, 's>,
    letter_query: LetterQuery<'w, 's>,
}

fn ui_tree(ui: &mut egui::Ui, mut params: TreeSystemParams) {
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
struct Openness(bool);

fn add_openness(mut commands: Commands, query: Query<Entity, Added<CircleChildren>>) {
    for entity in query.iter() {
        commands.entity(entity).insert(Openness(false));
    }
}
