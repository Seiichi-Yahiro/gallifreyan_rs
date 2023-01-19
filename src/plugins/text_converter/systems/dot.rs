use super::components::{Text, *};
use crate::utils::update_if_changed::update_if_changed;
use bevy::prelude::*;

pub fn convert_dots(
    mut commands: Commands,
    mut letter_query: Query<(Entity, &Letter, &Radius, &mut CircleChildren), Changed<Text>>,
    mut dot_query: Query<(Entity, &mut Radius, &mut PositionData), (With<Dot>, Without<Letter>)>,
) {
    for (letter_entity, letter, Radius(letter_radius), mut children) in letter_query.iter_mut() {
        let mut existing_dots = dot_query.iter_many_mut(children.iter());

        let number_of_dots = letter.dots();
        let mut new_dots_iter = 0..number_of_dots;

        let mut new_children: Vec<Entity> = Vec::with_capacity(number_of_dots);

        loop {
            let next_existing_dot = existing_dots.fetch_next();
            let next_new_dot = new_dots_iter.next();

            match (next_existing_dot, next_new_dot) {
                // update dot
                (Some((dot_entity, mut radius, mut position_data)), Some(_)) => {
                    let new_radius = Dot::radius(*letter_radius);
                    let new_position_data =
                        Dot::position_data(*letter_radius, number_of_dots, new_children.len());

                    update_if_changed!(**radius, new_radius, "Update dot radius: {} -> {}");

                    update_if_changed!(
                        *position_data,
                        new_position_data,
                        "Update dot position_data: {:?} -> {:?}"
                    );

                    new_children.push(dot_entity);
                }
                // remove dot
                (Some((dot_entity, _radius, _position_data)), None) => {
                    debug!("Despawn dot");
                    commands.entity(dot_entity).despawn_recursive();
                }
                // add dot
                (None, Some(_)) => {
                    debug!("Spawn dot");

                    let dot_bundle =
                        DotBundle::new(*letter_radius, number_of_dots, new_children.len());

                    let dot_entity = commands.spawn(dot_bundle).id();
                    commands.entity(letter_entity).add_child(dot_entity);
                    new_children.push(dot_entity);
                }
                (None, None) => {
                    break;
                }
            }
        }

        **children = new_children;
    }
}
