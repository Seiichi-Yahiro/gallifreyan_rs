use crate::sidebar::text_converter;
use crate::sidebar::text_converter::SetText;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::egui;

#[derive(Resource, Default)]
pub struct TextState {
    text: String,
    sanitized_text: String,
}

#[derive(SystemParam)]
pub struct TextInputSystemParams<'w, 's> {
    ui_state: ResMut<'w, TextState>,
    set_text_event: EventWriter<'w, 's, SetText>,
}

pub fn ui_text_input(ui: &mut egui::Ui, mut params: TextInputSystemParams) {
    let original_text_edit_width = ui.spacing().text_edit_width;
    ui.spacing_mut().text_edit_width = ui.available_width();

    if ui.text_edit_singleline(&mut params.ui_state.text).changed() {
        let new_sanitized_text = text_converter::sanitize_text_input(&params.ui_state.text);
        if params.ui_state.sanitized_text != new_sanitized_text {
            params.ui_state.sanitized_text = new_sanitized_text;
            params
                .set_text_event
                .send(SetText(params.ui_state.sanitized_text.clone()));
        }
    }

    ui.spacing_mut().text_edit_width = original_text_edit_width;
}
