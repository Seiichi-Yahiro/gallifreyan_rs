use crate::image_types::{AnglePlacement, LineSlot, PositionData, Radius};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::tess::path::path::Builder;
use bevy_prototype_lyon::prelude::*;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_radius)
            .add_system(update_position_data)
            .add_system(update_line_slot.after(update_position_data));
    }
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
        let translation = Vec3::new(0.0, -position_data.distance, 0.0);
        let rotation = Quat::from_rotation_z(position_data.angle.to_radians());

        match position_data.angle_placement {
            AnglePlacement::Absolute => {
                transform.translation = rotation * translation;
            }
            AnglePlacement::Relative => {
                *transform =
                    Transform::from_rotation(rotation) * Transform::from_translation(translation);
            }
        }
    }
}

fn update_line_slot(
    mut query: Query<(&mut Path, &Transform), (With<LineSlot>, Changed<PositionData>)>,
) {
    for (mut path, transform) in query.iter_mut() {
        let mut path_builder = Builder::new();

        let end = transform.translation.truncate().normalize_or_zero() * 10.0;
        let line = shapes::Line(Vec2::ZERO, end);

        line.add_geometry(&mut path_builder);

        *path = Path(path_builder.build());
    }
}
