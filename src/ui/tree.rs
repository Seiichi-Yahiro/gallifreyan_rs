use crate::event_set::SendEvent;
use crate::sidebar::{Actions, Hover, Select};
use bevy::prelude::Entity;
use bevy_egui::egui;

pub struct TreeNode<T> {
    pub id: T,
    pub text: String,
    pub open: bool,
    pub children: Vec<TreeNode<T>>,
}

impl TreeNode<Entity> {
    pub fn render(&mut self, ui: &mut egui::Ui, actions: &mut Actions) {
        if self.children.is_empty() {
            tree_item(ui, self.id, &self.text, actions);
        } else {
            let ui_id = ui.make_persistent_id(self.id);

            let mut collapsing_state =
                egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    ui_id,
                    false,
                );

            collapsing_state.set_open(self.open);

            let (collapse_response, _, _) = collapsing_state
                .show_header(ui, |ui| {
                    tree_item(ui, self.id, &self.text, actions);
                })
                .body(|ui| {
                    for child in &mut self.children {
                        child.render(ui, actions);
                    }
                });

            if collapse_response.clicked() {
                self.open = !self.open;
            }
        }
    }
}

fn tree_item(
    ui: &mut egui::Ui,
    id: Entity,
    text: impl Into<egui::WidgetText>,
    actions: &mut Actions,
) {
    let button = egui::Button::new(text).frame(false).wrap(true);
    let response = ui.add(button);

    if response.hovered() {
        actions.dispatch(Hover(id));
    }

    if response.clicked() {
        actions.dispatch(Select(id));
    }
}
