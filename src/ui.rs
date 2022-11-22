use crate::actions::{Actions, SetText, UiSizeChanged};
use crate::event_set::SendEvent;
use crate::text_converter;
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;
use bevy_egui::egui::Ui;
use bevy_egui::{egui, EguiContext, EguiPlugin};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiState::default())
            .add_plugin(EguiPlugin)
            .add_system(ui);
    }
}

#[derive(Default, Copy, Clone)]
pub struct UiSize {
    pub sidebar_width: f32,
}

#[derive(Resource, Default)]
pub struct UiState {
    pub text: String,
    pub sanitized_text: String,
    pub tree: Option<TreeNode>,
}

pub struct TreeNode {
    pub entity: Entity,
    pub text: String,
    pub open: bool,
    pub children: Vec<TreeNode>,
}

fn render_tree(node: &TreeNode, ui: &mut Ui) {
    egui::CollapsingHeader::new(&node.text)
        .id_source(node.entity)
        .default_open(node.open)
        .show(ui, |ui| {
            for child in &node.children {
                if child.children.is_empty() {
                    ui.label(&child.text);
                } else {
                    render_tree(child, ui);
                }
            }
        });
}

pub fn is_ui_blocking(mut egui_context: ResMut<EguiContext>) -> ShouldRun {
    let ctx = egui_context.ctx_mut();

    // somehow is_pointer_over_area always returns false when called in a run_criteria
    if ctx.wants_pointer_input() || ctx.wants_keyboard_input() {
        ShouldRun::No
    } else {
        ShouldRun::Yes
    }
}

pub fn ui(
    mut egui_context: ResMut<EguiContext>,
    mut ui_state: ResMut<UiState>,
    mut actions: Actions,
    mut local_ui_size: Local<UiSize>,
) {
    let side_bar =
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

    let side_bar_width = side_bar.response.rect.width();

    if side_bar_width != local_ui_size.sidebar_width {
        local_ui_size.sidebar_width = side_bar_width;
        actions.dispatch(UiSizeChanged(*local_ui_size));
    }
}
