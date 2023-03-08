use super::{UiBaseSet, UiSet};
use crate::plugins::svg_view::{CenterView, ViewMode};
use bevy::prelude::*;
use bevy_egui::egui::epaint::Shadow;
use bevy_egui::{egui, EguiContexts};

pub struct ToolBoxPlugin;

impl Plugin for ToolBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(ui.in_base_set(UiBaseSet).in_set(UiSet::Window));
    }
}

fn ui(
    mut egui_contexts: EguiContexts,
    current_view_mode: Res<State<ViewMode>>,
    mut next_view_mode: ResMut<NextState<ViewMode>>,
    mut center_view_events: EventWriter<CenterView>,
) {
    egui::Window::new("toolbox")
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .frame(
            egui::Frame::window(&egui_contexts.ctx_mut().style()).shadow(Shadow {
                extrusion: 0.0,
                color: egui::Color32::BLACK,
            }),
        )
        .fixed_size(egui::Vec2::new(20.0, 60.0))
        .anchor(egui::Align2::LEFT_TOP, egui::Vec2::splat(5.0))
        .show(egui_contexts.ctx_mut(), |ui| {
            ui.vertical_centered_justified(|ui| {
                if ui
                    .selectable_label(current_view_mode.0 == ViewMode::Select, "☝")
                    .on_hover_text("Select mode")
                    .clicked()
                {
                    next_view_mode.set(ViewMode::Select);
                }

                if ui
                    .selectable_label(current_view_mode.0 == ViewMode::Pan, "✋")
                    .on_hover_text("Pan mode")
                    .clicked()
                {
                    next_view_mode.set(ViewMode::Pan);
                }

                if ui.button("⛶").on_hover_text("Center view").clicked() {
                    center_view_events.send_default();
                }
            });
        });
}
