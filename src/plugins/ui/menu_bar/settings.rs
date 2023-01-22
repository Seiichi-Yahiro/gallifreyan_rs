pub mod vocal_nesting;

use super::UiStage;
use crate::plugins::color_theme::{ColorTheme, Theme};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::egui;
use std::marker::PhantomData;

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

// TODO remove phantom in new bevy version
#[derive(SystemParam)]
pub struct SettingsSystemParams<'w, 's> {
    color_theme: ResMut<'w, ColorTheme>,
    opened_setting_windows: ResMut<'w, OpenedSettingWindows>,
    #[system_param(ignore)]
    _phantom: PhantomData<&'s ()>,
}

pub fn ui(ui: &mut egui::Ui, mut params: SettingsSystemParams) {
    ui.menu_button("Settings", |ui| {
        let mut is_dark_theme = params.color_theme.current() == Theme::Dark;

        if ui.checkbox(&mut is_dark_theme, "Dark mode").changed() {
            let new_theme = if is_dark_theme {
                Theme::Dark
            } else {
                Theme::Light
            };

            params.color_theme.set_theme(new_theme, ui.ctx());
        }

        if ui.button("Vocal Nesting...").clicked() {
            params.opened_setting_windows.vocal_nesting = true;
            ui.close_menu();
        }
    });
}
