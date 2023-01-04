use crate::image_types::{Consonant, NestingSettings, Vocal};
use bevy::prelude::*;
use bevy::utils::hashbrown::HashSet;
use bevy_egui::{egui, EguiContext};
use strum::IntoEnumIterator;

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
                ui.label("Enter a comma separated list of a consonant followed by a vocal.");
                ui.label("You can use '*' as a wildcard.");

                if ui.text_edit_singleline(&mut *rules_string).changed() {
                    match parse_rules_string(&rules_string) {
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

fn parse_rules_string(rules: &str) -> Result<HashSet<(Consonant, Vocal)>, String> {
    let mut rules_map = HashSet::new();

    for rule in rules.split(',') {
        match rule.len() {
            0 => {
                return Err("Rule can't be empty!".to_string());
            }
            1 => {
                return if rule == "*" {
                    Err("Missing a consonant or vocal!".to_string())
                } else if Consonant::try_from(rule).is_ok() {
                    Err(format!("'{}' is missing a vocal!", rule))
                } else if Vocal::try_from(rule).is_ok() {
                    Err(format!("'{}' is missing a consonant!", rule))
                } else {
                    Err(format!("'{}' is not a valid letter!", rule))
                };
            }
            2 => {
                if Consonant::try_from(rule).is_ok() {
                    return Err(format!("'{}' is missing a vocal!", rule));
                }

                let (consonant, vocal) = rule.split_at(1);

                if consonant == "*" {
                    let vocal = Vocal::try_from(vocal)?;
                    for consonant in Consonant::iter() {
                        rules_map.insert((consonant, vocal));
                    }
                } else if vocal == "*" {
                    let consonant = Consonant::try_from(consonant)?;
                    for vocal in Vocal::iter() {
                        rules_map.insert((consonant, vocal));
                    }
                } else {
                    let consonant = Consonant::try_from(consonant)?;
                    let vocal = Vocal::try_from(vocal)?;
                    rules_map.insert((consonant, vocal));
                }
            }
            3 => {
                let (consonant, vocal) = rule.split_at(2);
                let consonant = Consonant::try_from(consonant)?;

                if vocal == "*" {
                    for vocal in Vocal::iter() {
                        rules_map.insert((consonant, vocal));
                    }
                } else {
                    let vocal = Vocal::try_from(vocal)?;
                    rules_map.insert((consonant, vocal));
                }
            }
            _ => {
                return Err(format!("Too many letters in '{}'!", rule));
            }
        }
    }

    Ok(rules_map)
}
