pub mod angle_slider;
pub mod tree;

use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;
use bevy_egui::EguiContext;

pub fn is_ui_blocking(mut egui_context: ResMut<EguiContext>) -> ShouldRun {
    let ctx = egui_context.ctx_mut();

    // somehow is_pointer_over_area always returns false when called in a run_criteria
    if ctx.wants_pointer_input() || ctx.wants_keyboard_input() {
        ShouldRun::No
    } else {
        ShouldRun::Yes
    }
}
