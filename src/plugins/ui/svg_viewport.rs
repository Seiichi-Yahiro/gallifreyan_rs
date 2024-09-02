use crate::plugins::svg::prelude::SVGElement;
use crate::plugins::ui::{styles, UiRoot};
use bevy::prelude::*;
use bevy::render::camera::Viewport;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;

pub struct SVGViewportPlugin;

impl Plugin for SVGViewportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup.in_set(UiSVGViewportSet))
            .add_systems(Update, update_svg_viewport_size)
            .observe(add_svg_viewport_render_layer);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct UiSVGViewportSet;

#[derive(Component)]
struct SVGViewport;

fn setup(mut commands: Commands, ui_root_query: Query<Entity, With<UiRoot>>) {
    let root = ui_root_query.get_single().unwrap();

    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.camera = Camera {
        order: 0,
        clear_color: ClearColorConfig::Custom(styles::BACKGROUND_COLOR),
        ..default()
    };

    commands.spawn((
        Name::new("SVG Viewport Camera"),
        SVGViewport,
        camera_bundle,
        SVG_VIEWPORT_RENDER_LAYER,
    ));

    commands
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
        .set_parent(root);
}

const SVG_VIEWPORT_RENDER_LAYER: RenderLayers = RenderLayers::layer(1);

fn add_svg_viewport_render_layer(trigger: Trigger<OnAdd, SVGElement>, mut commands: Commands) {
    debug!("Add svg viewport render layer for {:?}", trigger.entity());

    commands
        .entity(trigger.entity())
        .insert(SVG_VIEWPORT_RENDER_LAYER);
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
