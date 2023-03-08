use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;
use gallifreyan_lib::{math, plugins};

fn main() {
    let mut app = App::new();

    let mut default_plugins = DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Gallifreyan".to_string(),
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    });

    #[cfg(debug_assertions)]
    {
        default_plugins = default_plugins.set(LogPlugin {
            filter: "info,wgpu_core=warn,wgpu_hal=error,gallifreyan_lib=debug".into(),
            level: bevy::log::Level::DEBUG,
        });
    }

    app.insert_resource(Msaa::Sample4)
        .insert_resource(WinitSettings::desktop_app())
        .add_plugins(default_plugins)
        .add_plugin(ShapePlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(plugins::text_converter::TextConverterPlugin)
        .add_plugin(plugins::color_theme::ColorThemePlugin)
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
fn set_window_icon(
    winit_windows: NonSend<bevy::winit::WinitWindows>,
    bevy_windows: Query<Entity, With<Window>>,
) {
    let window_entity = bevy_windows
        .get_single()
        .expect("There should only be one window!");

    let primary = winit_windows.get_window(window_entity).unwrap();

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
