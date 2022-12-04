use bevy_egui::egui;
use bevy_egui::egui::collapsing_header::paint_default_icon;
use std::fmt::Debug;
use std::hash::Hash;

pub struct CollapsingTreeItem<'a, T: Hash + Debug> {
    id: T,
    text: &'a str,
    open: Option<&'a mut bool>,
    is_selected: bool,
    empty: bool,
}

impl<'a, T: Hash + Debug> CollapsingTreeItem<'a, T> {
    pub fn show<R>(
        mut self,
        ui: &mut egui::Ui,
        add_body: impl FnOnce(&mut egui::Ui) -> R,
    ) -> (egui::Response, Option<egui::InnerResponse<R>>) {
        let ui_id = ui.make_persistent_id(self.id);

        let mut collapsing_state = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            ui_id,
            false,
        );

        if let Some(open) = self.open.as_ref() {
            collapsing_state.set_open(**open);
        }

        let header_response = ui.horizontal(|ui| {
            if self.is_selected {
                ui.painter().rect_filled(
                    ui.max_rect(),
                    egui::Rounding::none(),
                    ui.style().visuals.selection.bg_fill,
                );
            }

            if self.empty {
                let size = egui::vec2(ui.spacing().indent, ui.spacing().icon_width);
                let (_id, rect) = ui.allocate_space(size);

                let visuals = ui.style().noninteractive();
                let fill_color = visuals.fg_stroke.color;

                ui.painter().circle_filled(rect.center(), 2.0, fill_color);
            } else {
                let toggle_button_response =
                    collapsing_state.show_toggle_button(ui, paint_default_icon);

                if toggle_button_response.clicked() {
                    if let Some(open) = self.open.as_mut() {
                        **open = !**open;
                    }
                }
            };

            let tree_item = TreeItem::new(self.text);
            ui.add(tree_item)
        });

        let tree_item_response = header_response.inner;

        let body_response =
            collapsing_state.show_body_indented(&header_response.response, ui, add_body);

        (tree_item_response, body_response)
    }
}

impl<'a, T: Hash + Debug> CollapsingTreeItem<'a, T> {
    pub fn new(text: &'a str, id: T, open: &'a mut bool, is_selected: bool) -> Self {
        Self {
            id,
            text,
            open: Some(open),
            is_selected,
            empty: false,
        }
    }

    pub fn new_empty(ui: &mut egui::Ui, text: &'a str, id: T, is_selected: bool) -> egui::Response {
        let (header_response, _) = Self {
            id,
            text,
            open: None,
            is_selected,
            empty: true,
        }
        .show(ui, |_| {});

        header_response
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
