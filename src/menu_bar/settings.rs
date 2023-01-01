use crate::style::{SetTheme, Styles, Theme};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::egui;

#[derive(SystemParam)]
pub struct SettingsSystemParams<'w, 's> {
    set_theme_event: EventWriter<'w, 's, SetTheme>,
    styles: Res<'w, Styles>,
}

pub fn ui(ui: &mut egui::Ui, mut params: SettingsSystemParams) {
    ui.menu_button("Settings", |ui| {
        let mut is_dark_theme = params.styles.theme == Theme::Dark;

        if ui.checkbox(&mut is_dark_theme, "Dark mode").changed() {
            let new_theme = if is_dark_theme {
                Theme::Dark
            } else {
                Theme::Light
            };
            params.set_theme_event.send(SetTheme(new_theme));
        }
    });
}
