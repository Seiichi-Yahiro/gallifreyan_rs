use crate::image_types::{
    ConsonantPlacement, Letter, LineSlot, PositionData, Radius, VocalPlacement,
};
use crate::math::angle::{Angle, Degree, Radian};
use crate::selection::Selected;
use crate::ui::angle_slider::AngleSlider;
use crate::update_if_changed::update_if_changed;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::egui;

#[derive(SystemParam)]
pub struct SelectionSystemParams<'w, 's> {
    selection_query: Query<
        'w,
        's,
        (
            Option<&'static Parent>,
            Option<&'static mut Radius>,
            &'static mut PositionData,
            Option<&'static Letter>,
            Option<&'static LineSlot>,
        ),
        With<Selected>,
    >,
    global_transform_query: Query<'w, 's, &'static GlobalTransform>,
}

pub fn ui_selection(ui: &mut egui::Ui, mut params: SelectionSystemParams) {
    let (parent, mut radius, mut position_data, letter, line_slot) =
        match params.selection_query.get_single_mut() {
            Ok(it) => it,
            Err(_) => {
                return;
            }
        };

    egui::TopBottomPanel::bottom("selection")
        .frame(egui::Frame::none())
        .show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                let original_slider_width = ui.spacing().slider_width;
                ui.spacing_mut().slider_width = ui.available_width();

                if let Some(radius) = &mut radius {
                    let new_radius = ui_radius(ui, ***radius);

                    update_if_changed!(***radius, new_radius, "Update radius: {} -> {}");
                }

                let can_change_distance = letter
                    .map(|letter| match letter {
                        Letter::Vocal(vocal) => {
                            VocalPlacement::from(*vocal) != VocalPlacement::OnLine
                        }
                        Letter::Consonant(consonant)
                        | Letter::ConsonantWithVocal { consonant, .. } => {
                            ConsonantPlacement::from(*consonant) != ConsonantPlacement::OnLine
                        }
                    })
                    .unwrap_or_else(|| line_slot.is_none());

                if can_change_distance {
                    let new_distance = ui_distance(ui, position_data.distance);

                    update_if_changed!(
                        position_data.distance,
                        new_distance,
                        "Update distance: {} -> {}"
                    );
                }

                ui.spacing_mut().slider_width /= 2.0;

                let new_angle = ui_angle(
                    ui,
                    position_data.angle,
                    &parent,
                    &params.global_transform_query,
                );

                update_if_changed!(position_data.angle, new_angle, "Update angle: {:?} -> {:?}");

                ui.spacing_mut().slider_width = original_slider_width;
            });
        });
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
    angle: Degree,
    parent: &Option<&Parent>,
    global_transform_query: &Query<&GlobalTransform>,
) -> Degree {
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
        .map(Radian::new)
        .map(Radian::to_degrees)
        .unwrap_or_default();

    let mut new_angle = angle;
    let angle = AngleSlider::new(
        &mut new_angle,
        Degree::new(0.0)..=Degree::new(360.0),
        angle_offset,
    ); // TODO angle constraints
    ui.add(angle);

    new_angle
}
