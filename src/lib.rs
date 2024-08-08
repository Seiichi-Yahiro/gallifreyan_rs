#[cfg(debug_assertions)]
mod debug;
mod plugins;

use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};
use bevy_prototype_lyon::prelude::ShapePlugin;
use std::time::Duration;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn run() {
    let mut app = App::new();

    #[allow(unused_mut)]
    let mut default_plugins = DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Gallifreyan".to_string(),
            fit_canvas_to_parent: true,
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    });

    #[cfg(target_arch = "wasm32")]
    {
        use bevy::log::{Level, LogPlugin};

        #[cfg(debug_assertions)]
        let level = "debug";

        #[cfg(not(debug_assertions))]
        let level = "warn";

        default_plugins = default_plugins.set(LogPlugin {
            filter: format!("wgpu=error,naga=warn,gallifreyan_lib={}", level),
            level: Level::INFO,
            custom_layer: |_| None,
        });
    }

    app.add_plugins(default_plugins)
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::reactive(Duration::from_secs_f64(1.0 / 15.0)),
            unfocused_mode: UpdateMode::reactive_low_power(Duration::from_secs(60)),
        })
        .add_plugins(ShapePlugin)
        .add_plugins((
            plugins::text_converter::TextConverterPlugin,
            plugins::ui::UiPlugin,
        ))
        .run();
}
