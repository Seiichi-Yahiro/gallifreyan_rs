use bevy_egui::egui;
use std::hash::Hash;

pub struct TreeNode<T: Hash + Copy> {
    pub id: T,
    pub text: String,
    pub open: bool,
    pub children: Vec<TreeNode<T>>,
}

pub fn render_tree<T: Hash + Copy>(node: &TreeNode<T>, ui: &mut egui::Ui) {
    egui::CollapsingHeader::new(&node.text)
        .id_source(node.id)
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
