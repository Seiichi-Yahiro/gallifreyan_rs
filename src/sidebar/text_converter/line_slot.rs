use crate::image_types::{Text, *};
use bevy::prelude::*;

pub fn convert_line_slots(
    mut commands: Commands,
    mut letter_query: Query<(Entity, &Letter, &Radius, &mut LineSlotChildren), Changed<Text>>,
    mut line_slot_query: Query<(Entity, &mut PositionData), With<LineSlot>>,
) {
    for (letter_entity, letter, Radius(letter_radius), mut children) in letter_query.iter_mut() {
        let mut existing_line_slots = line_slot_query.iter_many_mut(children.iter());

        let number_of_lines = letter.lines();
        let line_points_outside = match letter {
            Letter::Vocal(vocal) => VocalDecoration::from(*vocal) == VocalDecoration::LineOutside,
            Letter::Consonant(_) | Letter::ConsonantWithVocal { .. } => false,
        };
        let mut new_line_slots_iter = 0..number_of_lines;

        let mut new_children: Vec<Entity> = Vec::with_capacity(number_of_lines);

        loop {
            let next_existing_line_slot = existing_line_slots.fetch_next();
            let next_new_line_slot = new_line_slots_iter.next();

            match (next_existing_line_slot, next_new_line_slot) {
                // update line slot
                (Some((line_slot_entity, mut position_data)), Some(_)) => {
                    let new_position_data = LineSlot::position_data(
                        *letter_radius,
                        number_of_lines,
                        new_children.len(),
                        line_points_outside,
                    );

                    if *position_data != new_position_data {
                        *position_data = new_position_data;
                    }

                    new_children.push(line_slot_entity);
                }
                // remove line slot
                (Some((line_slot_entity, _position_data)), None) => {
                    commands.entity(line_slot_entity).despawn_recursive();
                }
                // add line slot
                (None, Some(_)) => {
                    let line_slot_bundle = LineSlotBundle::new(
                        *letter_radius,
                        number_of_lines,
                        new_children.len(),
                        line_points_outside,
                    );

                    let line_slot_entity = commands.spawn(line_slot_bundle).id();
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
