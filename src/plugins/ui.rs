mod interactions;
mod styles;
mod text_input;

use crate::plugins::ui::interactions::InteractionsPlugin;
use crate::plugins::ui::text_input::{TextInputPlugin, TextInputWidget};
use bevy::asset::load_internal_binary_asset;
use bevy::prelude::*;
use bevy::render::camera::Viewport;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;

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
            .add_plugins(TextInputPlugin)
            .add_systems(Startup, (setup_svg_viewport, setup_ui))
            .add_systems(Update, update_svg_viewport_size);
    }
}

#[derive(Component)]
struct SidePanel;

#[derive(Component)]
struct SVGViewport;

const SVG_VIEWPORT_RENDER_LAYER: RenderLayers = RenderLayers::layer(1);

fn setup_svg_viewport(mut commands: Commands) {
    commands.spawn((
        Name::new("SVG Viewport Camera"),
        SVGViewport,
        Camera2dBundle {
            camera: Camera {
                order: 0,
                clear_color: ClearColorConfig::Custom(styles::BACKGROUND_COLOR),
                ..default()
            },
            ..default()
        },
        SVG_VIEWPORT_RENDER_LAYER,
    ));

    // TODO remove
    use bevy_prototype_lyon::prelude::*;

    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&shapes::Circle {
                radius: 100.0,
                center: Vec2::new(0.0, 0.0),
            }),
            ..default()
        },
        Stroke::new(Color::WHITE, 5.0),
        SVG_VIEWPORT_RENDER_LAYER,
    ));
}

fn update_svg_viewport_size(
    viewport_query: Query<(&Node, &GlobalTransform), With<SVGViewport>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut svg_viewport_camera_query: Query<&mut Camera, With<SVGViewport>>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };

    let Ok((node, global_transform)) = viewport_query.get_single() else {
        return;
    };

    let rect = node.logical_rect(global_transform);

    let ww = window.resolution.physical_width() as f32;
    let wh = window.resolution.physical_height() as f32;
    let sf = window.resolution.scale_factor();

    let left = (rect.min.x * sf).clamp(0.0, ww);
    let top = (rect.min.y * sf).clamp(0.0, wh);
    let right = (ww - rect.max.x * sf).clamp(0.0, ww);
    let bottom = (wh - rect.max.y * sf).clamp(0.0, wh);
    let vw = (ww - left - right).max(1.0);
    let vh = (wh - top - bottom).max(1.0);

    let Ok(mut camera) = svg_viewport_camera_query.get_single_mut() else {
        return;
    };

    camera.viewport = Some(Viewport {
        physical_position: UVec2::new(left as u32, top as u32),
        physical_size: UVec2::new(vw as u32, vh as u32),
        ..default()
    });
}

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

    let root = commands
        .spawn((
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
        ))
        .id();

    let left = commands
        .spawn((
            Name::new("Ui Sidepanel"),
            SidePanel,
            NodeBundle {
                style: Style {
                    width: Val::Percent(20.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(styles::PADDING)),
                    border: UiRect::right(Val::Px(styles::BORDER_SIZE)),
                    ..default()
                },
                background_color: BackgroundColor(styles::BACKGROUND_COLOR),
                border_color: BorderColor(styles::BORDER_COLOR),
                ..default()
            },
        ))
        .set_parent(root)
        .id();

    let _svg_viewport = commands
        .spawn((
            Name::new("Ui SVG Viewport"),
            SVGViewport,
            NodeBundle {
                style: Style {
                    width: Val::Percent(80.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::NONE),
                ..default()
            },
        ))
        .set_parent(root)
        .id();

    commands
        .spawn(TextInputWidget::new(Some("Sentence".to_string())))
        .set_parent(left);
}
