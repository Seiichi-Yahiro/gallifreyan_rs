use crate::image_types::{LineSlot, PositionData, Radius};
use crate::math::Circle;
use bevy::prelude::*;

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::Last, update_circle_hitbox)
            .add_system_to_stage(CoreStage::Last, update_line_slot_hitbox);
    }
}

#[derive(Component)]
pub struct Interaction {
    pub hit_box: Box<dyn HitBox>,
    pub z: f32,
}

impl Interaction {
    pub fn new(hit_box: impl HitBox) -> Self {
        Self {
            hit_box: Box::new(hit_box),
            z: 0.0,
        }
    }
}

pub trait HitBox: Send + Sync + 'static {
    fn is_inside(&self, cursor_pos: Vec2) -> bool;
}

impl HitBox for Circle {
    fn is_inside(&self, cursor_pos: Vec2) -> bool {
        (self.position - cursor_pos).length() - self.radius <= 0.0
    }
}

fn update_circle_hitbox(
    mut with_radius_query: Query<
        (&mut Interaction, &Radius, &GlobalTransform),
        Or<(Changed<Radius>, Changed<GlobalTransform>)>,
    >,
) {
    for (mut interaction, radius, global_transform) in with_radius_query.iter_mut() {
        let translation = global_transform.translation();

        *interaction = Interaction {
            hit_box: Box::new(Circle {
                position: translation.truncate(),
                radius: **radius,
            }),
            z: translation.z,
        }
    }
}

fn update_line_slot_hitbox(
    mut line_slot_query: Query<
        (&mut Interaction, &GlobalTransform),
        (With<LineSlot>, Changed<GlobalTransform>),
    >,
) {
    for (mut interaction, global_transform) in line_slot_query.iter_mut() {
        let translation = global_transform.translation();

        *interaction = Interaction {
            hit_box: Box::new(Circle {
                position: translation.truncate(),
                radius: 5.0,
            }),
            z: translation.z,
        }
    }
}
