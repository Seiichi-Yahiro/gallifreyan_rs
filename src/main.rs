mod actions;
mod camera;
mod event_set;
mod image_types;
mod text_converter;
mod ui;

use crate::actions::ActionsPlugin;
use crate::camera::CameraPlugin;
use crate::image_types::{PositionData, Radius};
use crate::ui::UiPlugin;
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use bevy_prototype_lyon::prelude::tess::path::path::Builder;
use bevy_prototype_lyon::prelude::*;

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
        .add_startup_system(spawn_root)
        .add_plugin(ShapePlugin)
        .add_plugin(UiPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(ActionsPlugin::<Root>::default())
        .add_system(update_radius)
        .add_system(update_position_data)
        .run();
}

#[derive(Component)]
pub struct Root;

fn spawn_root(mut commands: Commands) {
    commands.spawn((SpatialBundle::default(), Root));
}

fn update_radius(mut query: Query<(&mut Path, &Radius), Changed<Radius>>) {
    for (mut path, radius) in query.iter_mut() {
        let mut path_builder = Builder::new();

        let circle = shapes::Circle {
            radius: **radius,
            center: Default::default(),
        };

        circle.add_geometry(&mut path_builder);

        *path = Path(path_builder.build());
    }
}

fn update_position_data(mut query: Query<(&mut Transform, &PositionData), Changed<PositionData>>) {
    for (mut transform, position_data) in query.iter_mut() {
        let (sin, cos) = (position_data.angle).to_radians().sin_cos();
        let v = Vec2::new(0.0, -position_data.distance);
        let translation = Vec3::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos, 0.0);
        transform.translation = translation;
    }
}
