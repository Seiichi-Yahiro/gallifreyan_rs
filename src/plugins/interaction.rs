use crate::math::Circle;
use crate::plugins::text_converter::components::{Dot, Letter, LineSlot, Radius, Sentence, Word};
use crate::plugins::text_converter::TextConverterBaseSet;
use bevy::prelude::*;

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(add_interaction.in_base_set(TextConverterBaseSet::PostTextConverter))
            .add_systems(
                (update_circle_hitbox, update_line_slot_hitbox).in_base_set(CoreSet::Last),
            );
    }
}

#[derive(Default, Component)]
pub struct Interaction {
    pub hit_box: Circle,
    pub z: f32,
}

impl Interaction {
    pub fn is_inside(&self, cursor_pos: Vec2) -> bool {
        (self.hit_box.position - cursor_pos).length() - self.hit_box.radius <= 0.0
    }
}

fn add_interaction(
    mut commands: Commands,
    query: Query<
        Entity,
        (
            Or<(
                Added<Sentence>,
                Added<Word>,
                Added<Letter>,
                Added<Dot>,
                Added<LineSlot>,
            )>,
            Without<Interaction>,
        ),
    >,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(Interaction::default());
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
            hit_box: Circle {
                position: translation.truncate(),
                radius: **radius,
            },
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
            hit_box: Circle {
                position: translation.truncate(),
                radius: 5.0,
            },
            z: translation.z,
        }
    }
}
