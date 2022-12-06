pub mod angle_slider;
pub mod tree;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_after(
            CoreStage::PreUpdate,
            UiStage,
            SystemStage::single_threaded(),
        );
    }
}

#[derive(StageLabel)]
pub struct UiStage;
