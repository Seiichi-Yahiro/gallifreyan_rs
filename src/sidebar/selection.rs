use crate::events::Selection;
use crate::image_types::{PositionData, Radius};
use crate::math::Angle;
use crate::ui::angle_slider::AngleSlider;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::egui;

#[derive(SystemParam)]
pub struct SelectionSystemParams<'w, 's> {
    selection: Res<'w, Selection>,
    selection_query: Query<
        'w,
        's,
        (
            Option<&'static Parent>,
            Option<&'static mut Radius>,
            &'static mut PositionData,
        ),
    >,
    global_transform_query: Query<'w, 's, &'static GlobalTransform>,
}

pub fn ui_selection(ui: &mut egui::Ui, mut params: SelectionSystemParams) {
    if let Some(selected_entity) = **params.selection {
        if let Ok((parent, mut radius, mut position_data)) =
            params.selection_query.get_mut(selected_entity)
        {
            egui::TopBottomPanel::bottom("selection")
                .frame(egui::Frame::none())
                .show_inside(ui, |ui| {
                    ui.separator();

                    ui.vertical_centered(|ui| {
                        let original_slider_width = ui.spacing().slider_width;
                        ui.spacing_mut().slider_width = ui.available_width();

                        if let Some(radius) = &mut radius {
                            let new_radius = ui_radius(ui, ***radius);

                            if new_radius != ***radius {
                                ***radius = new_radius;
                            }
                        }

                        let new_distance = ui_distance(ui, position_data.distance);

                        if new_distance != position_data.distance {
                            position_data.distance = new_distance;
                        }

                        ui.spacing_mut().slider_width /= 2.0;

                        let new_angle = ui_angle(
                            ui,
                            position_data.angle.as_degrees(),
                            &parent,
                            &params.global_transform_query,
                        );

                        if new_angle != position_data.angle.as_degrees() {
                            position_data.angle = Angle::new_degree(new_angle);
                        }

                        ui.spacing_mut().slider_width = original_slider_width;
                    });
                });
        }
    }
}

fn ui_radius(ui: &mut egui::Ui, radius: f32) -> f32 {
    ui.label("Radius");

    let mut new_radius = radius;
    let range = 0.0..=1000.0;
    let step = (range.end() - range.start()) as f64 / 100.0;

    let radius_slider = egui::Slider::new(&mut new_radius, range)
        .show_value(false)
        .step_by(step); // TODO radius constraints

    ui.add(radius_slider);

    new_radius
}

fn ui_distance(ui: &mut egui::Ui, distance: f32) -> f32 {
    ui.label("Distance");
    let mut new_distance = distance;

    let range = 0.0..=1000.0;
    let step = (range.end() - range.start()) / 100.0;

    let distance = egui::Slider::new(&mut new_distance, 0.0..=1000.0)
        .show_value(false)
        .step_by(step); // TODO distance constraints

    ui.add(distance);

    new_distance
}

fn ui_angle(
    ui: &mut egui::Ui,
    angle: f32,
    parent: &Option<&Parent>,
    global_transform_query: &Query<&GlobalTransform>,
) -> f32 {
    ui.label("Angle");

    let angle_offset = parent
        .map(|it| it.get())
        .and_then(|parent| global_transform_query.get(parent).ok())
        .map(|parent_global_transform| {
            parent_global_transform
                .compute_transform()
                .rotation
                .to_euler(EulerRot::XYZ)
                .2
        })
        .map(Angle::new_radian)
        .map(Angle::as_degrees)
        .unwrap_or(0.0);

    let mut new_angle = angle;
    let angle = AngleSlider::new(&mut new_angle, 0.0..=360.0, angle_offset); // TODO angle constraints
    ui.add(angle);

    new_angle
}
