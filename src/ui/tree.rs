use bevy_egui::egui;
use std::fmt::Debug;
use std::hash::Hash;

pub struct CollapsingTreeItem<'a, T: Hash + Debug> {
    id: T,
    text: &'a str,
    open: &'a mut bool,
}

impl<'a, T: Hash + Debug> CollapsingTreeItem<'a, T> {
    pub fn show<R>(
        self,
        ui: &mut egui::Ui,
        add_body: impl FnOnce(&mut egui::Ui) -> R,
    ) -> (
        egui::InnerResponse<egui::Response>,
        Option<egui::InnerResponse<R>>,
    ) {
        let ui_id = ui.make_persistent_id(self.id);

        let mut collapsing_state = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            ui_id,
            false,
        );

        collapsing_state.set_open(*self.open);

        let (collapse_response, header_response, body_response) = collapsing_state
            .show_header(ui, |ui| {
                let tree_item = TreeItem::new(self.text);
                ui.add(tree_item)
            })
            .body(add_body);

        if collapse_response.clicked() {
            *self.open = !*self.open;
        }

        (header_response, body_response)
    }
}

impl<'a, T: Hash + Debug> CollapsingTreeItem<'a, T> {
    pub fn new(text: &'a str, id: T, open: &'a mut bool) -> Self {
        Self { id, text, open }
    }
}

pub struct TreeItem<'a> {
    text: &'a str,
}

impl<'a> TreeItem<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }
}

impl<'a> egui::Widget for TreeItem<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let button = egui::Button::new(self.text).frame(false).wrap(true);
        ui.add(button)
    }
}
