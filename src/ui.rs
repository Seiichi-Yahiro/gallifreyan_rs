mod angle_slider;
mod sidebar;
mod tree;

pub use sidebar::UiSize;
pub use tree::TreeNode;

use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiPlugin};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiState::default())
            .add_plugin(EguiPlugin)
            .add_system_set(SystemSet::new().label(Ui).with_system(sidebar::ui));
    }
}

#[derive(SystemLabel)]
pub struct Ui;

#[derive(Resource, Default)]
pub struct UiState {
    pub text: String,
    pub sanitized_text: String,
    pub tree: Option<TreeNode>,
}

pub fn is_ui_blocking(mut egui_context: ResMut<EguiContext>) -> ShouldRun {
    let ctx = egui_context.ctx_mut();

    // somehow is_pointer_over_area always returns false when called in a run_criteria
    if ctx.wants_pointer_input() || ctx.wants_keyboard_input() {
        ShouldRun::No
    } else {
        ShouldRun::Yes
    }
}
