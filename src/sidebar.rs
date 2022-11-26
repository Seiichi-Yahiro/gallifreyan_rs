use crate::actions::{Actions, SetText};
use crate::event_set::SendEvent;
use crate::image_types::{CircleChildren, Letter, LineSlotChildren, Sentence, Text, Word};
use crate::text_converter;
use crate::ui::tree::{render_tree, TreeNode};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

pub struct SideBarPlugin;

impl Plugin for SideBarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SidebarState>()
            .add_system(ui)
            .add_system(build_tree);
    }
}

#[derive(Resource, Default)]
pub struct SidebarState {
    pub text: String,
    pub sanitized_text: String,
    pub tree: Option<TreeNode<Entity>>,
}

pub fn ui(
    mut egui_context: ResMut<EguiContext>,
    mut ui_state: ResMut<SidebarState>,
    mut actions: Actions,
) {
    egui::SidePanel::left("sidebar")
        .resizable(true)
        .show(egui_context.ctx_mut(), |ui| {
            if ui.text_edit_singleline(&mut ui_state.text).changed() {
                let new_sanitized_text = text_converter::sanitize_text_input(&ui_state.text);
                if ui_state.sanitized_text != new_sanitized_text {
                    ui_state.sanitized_text = new_sanitized_text;
                    actions.dispatch(SetText(ui_state.sanitized_text.clone()));
                }
            }

            if let Some(node) = &ui_state.tree {
                render_tree(node, ui);
            }
        });
}

fn build_tree(
    sentence_query: Query<&Sentence>,
    added_sentence_query: Query<(Entity, &Text, &CircleChildren), Added<Sentence>>,
    word_query: Query<(Entity, &Text, &CircleChildren), With<Word>>,
    letter_query: Query<(Entity, &Text, Option<&CircleChildren>, &LineSlotChildren), With<Letter>>,
    mut ui_state: ResMut<SidebarState>,
) {
    if let Ok((sentence, sentence_text, words)) = added_sentence_query.get_single() {
        let sentence_children = word_query
            .iter_many(words.iter())
            .map(|(word, word_text, letters)| {
                let word_children = letter_query
                    .iter_many(letters.iter())
                    .map(|(letter, letter_text, dots, letter_line_slots)| TreeNode {
                        id: letter,
                        text: letter_text.to_string(),
                        open: false,
                        children: dots
                            .map(|dots| {
                                dots.iter()
                                    .map(|dot| TreeNode {
                                        id: *dot,
                                        text: "DOT".to_string(),
                                        open: false,
                                        children: vec![],
                                    })
                                    .chain(letter_line_slots.iter().map(|line_slot| TreeNode {
                                        id: *line_slot,
                                        text: "LINE".to_string(),
                                        open: false,
                                        children: vec![],
                                    }))
                                    .collect::<Vec<TreeNode<Entity>>>()
                            })
                            .unwrap_or_default(),
                    })
                    .collect();

                TreeNode {
                    id: word,
                    text: word_text.to_string(),
                    open: false,
                    children: word_children,
                }
            })
            .collect();

        let sentence_node = TreeNode {
            id: sentence,
            text: sentence_text.to_string(),
            open: true,
            children: sentence_children,
        };

        ui_state.tree = Some(sentence_node);
    } else if sentence_query.iter().next().is_none() {
        ui_state.tree = None;
    }
}
