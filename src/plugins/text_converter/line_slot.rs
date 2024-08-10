use super::components::{Text, *};
use crate::plugins::text_converter::letter::Decorated;
use bevy::prelude::*;

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct LineSlot;

#[derive(Bundle)]
pub struct LineSlotBundle {
    pub name: Name,
    pub line_slot: LineSlot,
}

impl LineSlotBundle {
    pub fn new() -> Self {
        Self {
            name: Name::new("Line Slot"),
            line_slot: LineSlot,
        }
    }
}

pub fn convert_line_slots(
    mut commands: Commands,
    mut letter_query: Query<(Entity, &Letter, &mut LineSlotChildren), Changed<Text>>,
    mut line_slot_query: Query<Entity, With<LineSlot>>,
) {
    for (letter_entity, letter, mut children) in letter_query.iter_mut() {
        let mut existing_line_slots = line_slot_query.iter_many_mut(children.iter());

        let number_of_lines = letter.lines();
        let mut new_line_slots_iter = 0..number_of_lines;

        let mut new_children: Vec<Entity> = Vec::with_capacity(number_of_lines);

        loop {
            let next_existing_line_slot = existing_line_slots.fetch_next();
            let next_new_line_slot = new_line_slots_iter.next();

            match (next_existing_line_slot, next_new_line_slot) {
                // update line slot
                (Some(line_slot_entity), Some(_)) => {
                    new_children.push(line_slot_entity);
                }
                // remove line slot
                (Some(line_slot_entity), None) => {
                    debug!("Despawn line_slot");
                    commands.entity(line_slot_entity).despawn_recursive();
                }
                // add line slot
                (None, Some(_)) => {
                    debug!("Spawn line_slot");

                    let bundle = LineSlotBundle::new();

                    let line_slot_entity = commands.spawn(bundle).id();
                    commands.entity(letter_entity).add_child(line_slot_entity);
                    new_children.push(line_slot_entity);
                }
                (None, None) => {
                    break;
                }
            }
        }

        **children = new_children;
    }
}
