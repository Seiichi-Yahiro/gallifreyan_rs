pub mod vocal_nesting;

use super::UiStage;
use crate::plugins::style::{SetTheme, Styles, Theme};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::egui;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OpenedSettingWindows>()
            .add_system_to_stage(
                UiStage,
                vocal_nesting::ui.after(super::super::sidebar::UiSystemLabel),
            );
    }
}

#[derive(Default, Resource)]
pub struct OpenedSettingWindows {
    vocal_nesting: bool,
}

#[derive(SystemParam)]
pub struct SettingsSystemParams<'w, 's> {
    set_theme_event: EventWriter<'w, 's, SetTheme>,
    styles: Res<'w, Styles>,
    opened_setting_windows: ResMut<'w, OpenedSettingWindows>,
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

        if ui.button("Vocal Nesting...").clicked() {
            params.opened_setting_windows.vocal_nesting = true;
            ui.close_menu();
        }
    });
}
