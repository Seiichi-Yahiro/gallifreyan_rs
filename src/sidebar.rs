mod text_converter;

use crate::event_set::*;
use crate::image_types::{CircleChildren, LineSlotChildren, Sentence, Text};
use crate::sidebar::text_converter::{SetText, TextConverterPlugin};
use crate::ui::tree::TreeNode;
use bevy::ecs::query::QuerySingleError;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

pub struct SideBarPlugin;

impl Plugin for SideBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_event_set::<Actions>()
            .init_resource::<SidebarState>()
            .add_system(ui)
            .add_system_to_stage(CoreStage::PostUpdate, build_tree)
            .add_plugin(TextConverterPlugin);
    }
}

event_set!(Actions { Select, Hover });

pub struct Select(pub Entity);
pub struct Hover(pub Entity);

#[derive(Resource, Default)]
pub struct SidebarState {
    pub text: String,
    pub sanitized_text: String,
    pub tree: Option<TreeNode<Entity>>,
}

pub fn ui(
    mut egui_context: ResMut<EguiContext>,
    mut ui_state: ResMut<SidebarState>,
    mut set_text_event: EventWriter<SetText>,
    mut actions: Actions,
) {
    egui::SidePanel::left("sidebar")
        .resizable(true)
        .show(egui_context.ctx_mut(), |ui| {
            if ui.text_edit_singleline(&mut ui_state.text).changed() {
                let new_sanitized_text = text_converter::sanitize_text_input(&ui_state.text);
                if ui_state.sanitized_text != new_sanitized_text {
                    ui_state.sanitized_text = new_sanitized_text;
                    set_text_event.send(SetText(ui_state.sanitized_text.clone()));
                }
            }

            if let Some(node) = &mut ui_state.tree {
                node.render(ui, &mut actions);
            }
        });
}

fn build_tree(
    changed_query: Query<
        Entity,
        Or<(
            Changed<Text>,
            Changed<CircleChildren>,
            Changed<LineSlotChildren>,
        )>,
    >,
    sentence_query: Query<Entity, With<Sentence>>,
    text_query: Query<(Entity, &Text, &CircleChildren, &LineSlotChildren)>,
    mut ui_state: ResMut<SidebarState>,
) {
    match sentence_query.get_single() {
        Ok(sentence_entity) if changed_query.iter().last().is_some() => {
            if let Ok((_, sentence_text, words, sentence_line_slots)) =
                text_query.get(sentence_entity)
            {
                let map_line_slots = |line_slot: &Entity| TreeNode {
                    id: *line_slot,
                    text: "LINE".to_string(),
                    open: false,
                    children: vec![],
                };

                let children = text_query
                    .iter_many(words.iter())
                    .map(|(word_entity, word_text, letters, word_line_slots)| {
                        let children = text_query
                            .iter_many(letters.iter())
                            .map(|(letter_entity, letter_text, dots, letter_line_slots)| {
                                let children = dots
                                    .iter()
                                    .map(|dot| TreeNode {
                                        id: *dot,
                                        text: "DOT".to_string(),
                                        open: false,
                                        children: vec![],
                                    })
                                    .chain(letter_line_slots.iter().map(map_line_slots))
                                    .collect();

                                TreeNode {
                                    id: letter_entity,
                                    text: letter_text.to_string(),
                                    open: false,
                                    children,
                                }
                            })
                            .chain(word_line_slots.iter().map(map_line_slots))
                            .collect();

                        TreeNode {
                            id: word_entity,
                            text: word_text.to_string(),
                            open: false,
                            children,
                        }
                    })
                    .chain(sentence_line_slots.iter().map(map_line_slots))
                    .collect();

                ui_state.tree = Some(TreeNode {
                    id: sentence_entity,
                    text: sentence_text.to_string(),
                    open: true,
                    children,
                });
            }
        }
        Err(QuerySingleError::NoEntities(_)) => {
            ui_state.tree = None;
        }
        error => {
            error.unwrap();
        }
    }
}
