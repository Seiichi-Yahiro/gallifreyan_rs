mod menu_bar;
mod sidebar;
mod toolbox;
mod widgets;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_after(
            CoreStage::PreUpdate,
            UiStage,
            SystemStage::single_threaded(),
        )
        .add_plugin(menu_bar::MenuBarPlugin)
        .add_plugin(sidebar::SideBarPlugin)
        .add_plugin(toolbox::ToolBoxPlugin);
    }
}

#[derive(StageLabel)]
pub struct UiStage;
