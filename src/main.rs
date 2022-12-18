#![feature(option_result_contains)]

mod event_set;
mod events;
mod image_types;
mod math;
mod menu_bar;
mod sidebar;
mod style;
mod svg;
mod svg_view;
mod ui;

use crate::events::EventPlugin;
use crate::menu_bar::MenuBarPlugin;
use crate::sidebar::SideBarPlugin;
use crate::style::StylePlugin;
use crate::svg_view::SVGViewPlugin;
use crate::ui::UiPlugin;
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
                fit_canvas_to_parent: true,
                ..default()
            },
            ..default()
        }))
        .add_plugin(ShapePlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(StylePlugin)
        .add_plugin(EventPlugin)
        .add_plugin(MenuBarPlugin)
        .add_plugin(SideBarPlugin)
        .add_plugin(SVGViewPlugin)
        .register_type::<image_types::Sentence>()
        .register_type::<image_types::Word>()
        .register_type::<image_types::Letter>()
        .register_type::<image_types::Dot>()
        .register_type::<image_types::LineSlot>()
        .register_type::<image_types::CircleChildren>()
        .register_type::<image_types::LineSlotChildren>()
        .register_type::<image_types::Text>()
        .register_type::<image_types::Radius>()
        .register_type::<image_types::PositionData>()
        .register_type::<image_types::AnglePlacement>()
        .register_type::<image_types::Placement>()
        .register_type::<image_types::Decoration>()
        .register_type::<svg_view::Interaction>()
        .register_type::<math::Circle>()
        .register_type::<math::Angle>()
        .run();
}
