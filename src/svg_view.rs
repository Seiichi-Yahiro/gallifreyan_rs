mod camera;

use crate::svg_view::camera::SVGViewCamera;
use bevy::prelude::*;
use bevy_egui::egui::epaint::Shadow;
use bevy_egui::{egui, EguiContext};

pub struct SVGViewPlugin;

impl Plugin for SVGViewPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ViewMode::Select)
            .add_plugin(camera::CameraPlugin)
            .add_system(ui.after(crate::sidebar::UiSystemLabel));
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Resource)]
pub enum ViewMode {
    Select,
    Pan,
}

fn ui(
    mut egui_context: ResMut<EguiContext>,
    mut view_mode: ResMut<ViewMode>,
    mut camera_query: Query<(&mut OrthographicProjection, &mut Transform), With<SVGViewCamera>>,
) {
    egui::Window::new("svg controls")
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .frame(
            egui::Frame::window(&egui_context.ctx_mut().style()).shadow(Shadow {
                extrusion: 0.0,
                color: egui::Color32::BLACK,
            }),
        )
        .fixed_size(egui::Vec2::new(20.0, 60.0))
        .anchor(egui::Align2::LEFT_TOP, egui::Vec2::splat(5.0))
        .show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered_justified(|ui| {
                if ui
                    .selectable_label(*view_mode == ViewMode::Select, "☝")
                    .on_hover_text("Select mode")
                    .clicked()
                {
                    *view_mode = ViewMode::Select;
                }

                if ui
                    .selectable_label(*view_mode == ViewMode::Pan, "✋")
                    .on_hover_text("Pan mode")
                    .clicked()
                {
                    *view_mode = ViewMode::Pan;
                }

                if ui.button("⛶").on_hover_text("Center view").clicked() {
                    let (mut orthographic_projection, mut transform) = camera_query.single_mut();

                    orthographic_projection.scale = 1.0;
                    transform.translation = Vec3::new(0.0, 0.0, transform.translation.z);
                }
            });
        });
}
