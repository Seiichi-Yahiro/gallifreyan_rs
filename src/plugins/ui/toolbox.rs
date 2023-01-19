use super::UiStage;
use crate::plugins::svg_view::{CenterView, ViewMode};
use bevy::prelude::*;
use bevy_egui::egui::epaint::Shadow;
use bevy_egui::{egui, EguiContext};

pub struct ToolBoxPlugin;

impl Plugin for ToolBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(UiStage, ui.after(super::sidebar::UiSystemLabel));
    }
}

fn ui(
    mut egui_context: ResMut<EguiContext>,
    mut view_mode: ResMut<State<ViewMode>>,
    mut center_view_events: EventWriter<CenterView>,
) {
    egui::Window::new("toolbox")
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
                let current_view_mode = *view_mode.current();

                if ui
                    .selectable_label(current_view_mode == ViewMode::Select, "☝")
                    .on_hover_text("Select mode")
                    .clicked()
                {
                    view_mode.set(ViewMode::Select).ok();
                }

                if ui
                    .selectable_label(current_view_mode == ViewMode::Pan, "✋")
                    .on_hover_text("Pan mode")
                    .clicked()
                {
                    view_mode.set(ViewMode::Pan).ok();
                }

                if ui.button("⛶").on_hover_text("Center view").clicked() {
                    center_view_events.send_default();
                }
            });
        });
}
