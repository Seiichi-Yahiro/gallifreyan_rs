mod icons;
mod interactions;
mod sidebar;
mod styles;
mod svg_viewport;
mod widgets;

use crate::plugins::ui::icons::IconsPlugin;
use crate::plugins::ui::interactions::InteractionsPlugin;
use bevy::asset::load_internal_binary_asset;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        load_internal_binary_asset!(
            app,
            styles::FONT_HANDLE,
            "../../assets/Roboto-Regular.ttf",
            |bytes: &[u8], _path: String| { Font::try_from_bytes(bytes.to_vec()).unwrap() }
        );

        app.add_plugins(InteractionsPlugin)
            .add_plugins((
                IconsPlugin,
                widgets::text_input::TextInputPlugin,
                widgets::scroll_area::ScrollAreaPlugin,
            ))
            .add_systems(Startup, setup_ui)
            .configure_sets(
                Startup,
                (sidebar::UiSidebarSet, svg_viewport::UiSVGViewportSet)
                    .chain()
                    .after(setup_ui),
            )
            .add_plugins((sidebar::SidebarPlugin, svg_viewport::SVGViewportPlugin));
    }
}

#[derive(Component)]
struct UiRoot;

fn setup_ui(mut commands: Commands) {
    let ui_camera = commands
        .spawn((
            Name::new("Ui Camera"),
            Camera2dBundle {
                camera: Camera {
                    order: 1,
                    clear_color: ClearColorConfig::None,
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    commands.spawn((
        Name::new("Ui root"),
        UiRoot,
        TargetCamera(ui_camera),
        NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            background_color: BackgroundColor(Color::NONE),
            ..default()
        },
    ));
}
