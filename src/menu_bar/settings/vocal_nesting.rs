use crate::image_types::{Consonant, NestingSettings, Vocal};
use bevy::prelude::*;
use bevy::utils::hashbrown::HashSet;
use bevy_egui::{egui, EguiContext};

pub fn ui(
    mut egui_context: ResMut<EguiContext>,
    mut opened_setting_windows: ResMut<super::OpenedSettingWindows>,
    mut nesting_settings: ResMut<NestingSettings>,
    mut rules: Local<Option<HashSet<(Consonant, Vocal)>>>,
    mut rules_string: Local<String>,
    mut error_message: Local<Option<String>>,
) {
    egui::Window::new("Vocal Nesting")
        .open(&mut opened_setting_windows.vocal_nesting)
        .show(egui_context.ctx_mut(), |ui| {
            if ui
                .radio(matches!(*nesting_settings, NestingSettings::None), "None")
                .clicked()
            {
                if let NestingSettings::Custom(custom_rules) =
                    std::mem::replace(&mut *nesting_settings, NestingSettings::None)
                {
                    *rules = Some(custom_rules);
                }
            }

            if ui
                .radio(matches!(*nesting_settings, NestingSettings::All), "All")
                .clicked()
            {
                if let NestingSettings::Custom(custom_rules) =
                    std::mem::replace(&mut *nesting_settings, NestingSettings::All)
                {
                    *rules = Some(custom_rules);
                }
            }

            let is_custom = matches!(*nesting_settings, NestingSettings::Custom(_));

            if ui.radio(is_custom, "Custom").clicked() {
                if let Some(rules) = rules.take() {
                    *nesting_settings = NestingSettings::Custom(rules);
                } else if !is_custom {
                    *nesting_settings = NestingSettings::Custom(HashSet::new());
                }
            }

            ui.add_enabled_ui(is_custom, |ui| {
                ui.label("Comma separated list of consonants followed by a vocal:");

                if ui.text_edit_singleline(&mut *rules_string).changed() {
                    match convert_rules_string(&rules_string) {
                        Ok(new_rules) => {
                            *nesting_settings = NestingSettings::Custom(new_rules);
                            *error_message = None;
                        }
                        Err(error) => {
                            *error_message = Some(error);
                        }
                    }
                }

                if let Some(msg) = error_message.as_ref() {
                    ui.label(msg);
                }
            });
        });
}

fn convert_rules_string(rules: &str) -> Result<HashSet<(Consonant, Vocal)>, String> {
    let mut rules_map = HashSet::new();

    for rule in rules.split(',') {
        if rule.is_empty() {
            return Err("Rule can't be empty!".to_string());
        } else if rule.len() == 1 {
            Consonant::try_from(rule)?;
            return Err("Not enough letters!".to_string());
        } else {
            let (consonant, vocal) = rule.split_at(rule.len() - 1);

            let consonant = Consonant::try_from(consonant)?;
            let vocal = Vocal::try_from(vocal)?;

            rules_map.insert((consonant, vocal));
        }
    }

    Ok(rules_map)
}
