#![feature(option_result_contains)]

mod math;
mod plugins;
mod utils;

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
        .add_plugin(plugins::text_converter::TextConverterPlugin)
        .add_plugin(plugins::style::StylePlugin)
        .add_plugin(plugins::ui::UiPlugin)
        .add_plugin(plugins::svg_view::SVGViewPlugin)
        .add_plugin(plugins::svg::SVGPlugin)
        .add_plugin(plugins::interaction::InteractionPlugin)
        .add_plugin(plugins::selection::SelectionPlugin)
        .add_plugin(plugins::file::FilePlugin)
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
