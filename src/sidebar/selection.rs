use crate::events::Selection;
use crate::image_types::{PositionData, Radius};
use crate::ui::angle_slider::AngleSlider;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::egui;

#[derive(SystemParam)]
pub struct SelectionSystemParams<'w, 's> {
    selection: Res<'w, Selection>,
    selection_query: Query<'w, 's, (Option<&'static mut Radius>, &'static mut PositionData)>,
}

pub fn ui_selection(ui: &mut egui::Ui, mut params: SelectionSystemParams) {
    if let Some(selected_entity) = **params.selection {
        if let Ok((mut radius, mut position_data)) = params.selection_query.get_mut(selected_entity)
        {
            egui::TopBottomPanel::bottom("selection")
                .frame(egui::Frame::none())
                .show_inside(ui, |ui| {
                    ui.separator();

                    ui.vertical_centered(|ui| {
                        let original_slider_width = ui.spacing().slider_width;
                        ui.spacing_mut().slider_width = ui.available_width();

                        if let Some(radius) = &mut radius {
                            ui.label("Radius");

                            let mut new_radius: f32 = ***radius;
                            let range = 0.0..=1000.0;
                            let step = (range.end() - range.start()) as f64 / 100.0;

                            let radius_slider = egui::Slider::new(&mut new_radius, range)
                                .show_value(false)
                                .step_by(step); // TODO radius constraints

                            ui.add(radius_slider);

                            if new_radius != ***radius {
                                ***radius = new_radius;
                            }
                        }

                        ui.label("Distance");
                        let mut new_distance = position_data.distance;

                        let range = 0.0..=1000.0;
                        let step = (range.end() - range.start()) as f64 / 100.0;

                        let distance = egui::Slider::new(&mut new_distance, 0.0..=1000.0)
                            .show_value(false)
                            .step_by(step); // TODO distance constraints

                        ui.add(distance);

                        if new_distance != position_data.distance {
                            position_data.distance = new_distance;
                        }

                        ui.spacing_mut().slider_width /= 2.0;

                        ui.label("Angle");
                        let mut new_angle = position_data.angle;
                        let angle = AngleSlider::new(&mut new_angle, 0.0..=360.0); // TODO angle constraints
                        ui.add(angle);

                        if new_angle != position_data.angle {
                            position_data.angle = new_angle;
                        }

                        ui.spacing_mut().slider_width = original_slider_width;
                    });
                });
        }
    }
}
