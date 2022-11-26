use bevy::prelude::*;
use bevy_egui::egui;

pub struct TreeNode {
    pub entity: Entity,
    pub text: String,
    pub open: bool,
    pub children: Vec<TreeNode>,
}

pub fn render_tree(node: &TreeNode, ui: &mut egui::Ui) {
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
