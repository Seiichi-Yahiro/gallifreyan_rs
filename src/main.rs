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
        .add_plugin(ShapePlugin)
        .add_plugin(UiPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(ActionsPlugin)
        .add_system(geometry_builder)
        .run();
}

fn geometry_builder(
    mut commands: Commands,
    query: Query<(Entity, &PositionData, &Radius), (Added<PositionData>, Added<Radius>)>,
) {
    for (entity, position_data, radius) in query.iter() {
        let circle = shapes::Circle {
            radius: **radius,
            center: Default::default(),
        };
        let shape = GeometryBuilder::build_as(
            &circle,
            DrawMode::Stroke(StrokeMode::new(Color::BLACK, 4.0)),
            Default::default(),
        );

        commands.entity(entity).insert(shape);
    }
}
