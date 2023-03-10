mod menu_bar;
mod sidebar;
mod toolbox;
mod widgets;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.configure_set(UiBaseSet.after(CoreSet::PreUpdate).before(CoreSet::Update))
            .configure_sets((UiSet::MenuBar, UiSet::SideBar, UiSet::Window).chain())
            .add_plugin(menu_bar::MenuBarPlugin)
            .add_plugin(sidebar::SideBarPlugin)
            .add_plugin(toolbox::ToolBoxPlugin);
    }
}

#[derive(SystemSet, Debug, Eq, PartialEq, Copy, Clone, Hash)]
#[system_set(base)]
pub struct UiBaseSet;

#[derive(SystemSet, Debug, Eq, PartialEq, Copy, Clone, Hash)]
enum UiSet {
    MenuBar,
    SideBar,
    Window,
}
