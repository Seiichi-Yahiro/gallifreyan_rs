#![feature(option_result_contains)]

mod event_set;
mod image_types;
mod math;
mod menu_bar;
mod selection;
mod sidebar;
mod style;
mod svg_builder;
mod svg_view;
mod ui;

use crate::menu_bar::MenuBarPlugin;
use crate::selection::EventPlugin;
use crate::sidebar::SideBarPlugin;
use crate::style::StylePlugin;
use crate::svg_view::SVGViewPlugin;
use crate::ui::UiPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;

fn main() {
    let mut app = App::new();

    let mut default_plugins = DefaultPlugins.set(WindowPlugin {
        window: WindowDescriptor {
            title: "Gallifreyan".to_string(),
            fit_canvas_to_parent: true,
            ..default()
        },
        ..default()
    });

    #[cfg(debug_assertions)]
    {
        default_plugins = default_plugins.set(LogPlugin {
            filter: "info,wgpu_core=warn,wgpu_hal=error,gallifreyan_rs=debug".into(),
            level: bevy::log::Level::DEBUG,
        });
    }

    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(WinitSettings::desktop_app())
        .add_plugins(default_plugins)
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
        .register_type::<image_types::Consonant>()
        .register_type::<image_types::Vocal>()
        .register_type::<image_types::NestedLetter>()
        .register_type::<image_types::NestedVocal>()
        .register_type::<image_types::NestedVocalPositionCorrection>()
        .register_type::<Option<Entity>>()
        .register_type::<image_types::Dot>()
        .register_type::<image_types::LineSlot>()
        .register_type::<image_types::CircleChildren>()
        .register_type::<image_types::LineSlotChildren>()
        .register_type::<image_types::Text>()
        .register_type::<image_types::Radius>()
        .register_type::<image_types::PositionData>()
        .register_type::<image_types::AnglePlacement>()
        .register_type::<svg_view::Interaction>()
        .register_type::<math::Circle>()
        .register_type::<math::angle::Degree>();

    #[cfg(not(target_arch = "wasm32"))]
    app.add_startup_system(set_window_icon);

    app.run();
}

#[cfg(not(target_arch = "wasm32"))]
fn set_window_icon(windows: NonSend<bevy::winit::WinitWindows>) {
    let primary = windows
        .get_window(bevy::window::WindowId::primary())
        .unwrap();

    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(include_bytes!("../wasm/favicon-32x32.png"))
            .unwrap()
            .into_rgba8();

        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    let icon = winit::window::Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    primary.set_window_icon(Some(icon));
}
