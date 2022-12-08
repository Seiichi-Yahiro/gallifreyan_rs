mod event_set;
mod events;
mod image_types;
mod math;
mod sidebar;
mod style;
mod svg_view;
mod ui;
mod world_cursor;

use crate::events::EventPlugin;
use crate::sidebar::SideBarPlugin;
use crate::style::StylePlugin;
use crate::svg_view::SVGViewPlugin;
use crate::ui::UiPlugin;
use crate::world_cursor::WorldCursorPlugin;
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WinitSettings::desktop_app())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Gallifreyan".to_string(),
                ..default()
            },
            ..default()
        }))
        .add_plugin(ShapePlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(StylePlugin)
        .add_plugin(WorldCursorPlugin)
        .add_plugin(EventPlugin)
        .add_plugin(SideBarPlugin)
        .add_plugin(SVGViewPlugin)
        .run();
}
