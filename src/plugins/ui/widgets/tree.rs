use bevy_egui::egui;
use bevy_egui::egui::collapsing_header::paint_default_icon;
use std::fmt::Debug;
use std::hash::Hash;

pub struct CollapsingTreeItem<'a, T: Copy + Hash + Debug> {
    id: T,
    text: &'a str,
    open: Option<&'a mut bool>,
    is_selected: bool,
    empty: bool,
}

impl<'a, T: Copy + Hash + Debug> CollapsingTreeItem<'a, T> {
    pub fn show<R>(
        mut self,
        ui: &mut egui::Ui,
        add_body: impl FnOnce(&mut egui::Ui) -> R,
    ) -> (egui::Response, Option<egui::InnerResponse<R>>) {
        let collapsing_id = ui.make_persistent_id(self.id);

        let mut collapsing_state = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            collapsing_id,
            false,
        );

        if let Some(open) = self.open.as_ref() {
            collapsing_state.set_open(**open);
        }

        let hover_id = ui.make_persistent_id(format!("{:?}_hover", self.id));
        let is_hovered = ui
            .ctx()
            .data_mut(|it| it.get_temp::<bool>(hover_id))
            .unwrap_or_default();

        let frame = egui::Frame {
            inner_margin: Default::default(),
            outer_margin: Default::default(),
            rounding: ui.visuals().widgets.hovered.rounding,
            shadow: Default::default(),
            fill: if self.is_selected {
                ui.visuals().selection.bg_fill
            } else if is_hovered {
                ui.visuals().widgets.hovered.bg_fill
            } else {
                Default::default()
            },
            stroke: Default::default(),
        };

        let header_response = frame.show(ui, |ui| {
            ui.horizontal(|ui| {
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

                let available_width = ui.available_width();

                let galley = egui::WidgetText::from(self.text).into_galley(
                    ui,
                    Some(true),
                    available_width,
                    egui::TextStyle::Button,
                );

                let header_text_response = ui.allocate_response(
                    egui::Vec2::new(available_width, galley.size().y),
                    egui::Sense::click(),
                );

                ui.allocate_ui_at_rect(header_text_response.rect, |ui| {
                    let visuals = ui.style().interact(&header_text_response);
                    let pos = header_text_response.rect.left_top();
                    galley.paint_with_visuals(ui.painter(), pos, visuals);
                });

                header_text_response
            })
            .inner
        });

        ui.ctx()
            .data_mut(|it| it.insert_temp(hover_id, header_response.response.hovered()));

        let header_text_response = header_response.inner;

        let body_response =
            collapsing_state.show_body_indented(&header_response.response, ui, add_body);

        (header_text_response, body_response)
    }
}

impl<'a, T: Copy + Hash + Debug> CollapsingTreeItem<'a, T> {
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
