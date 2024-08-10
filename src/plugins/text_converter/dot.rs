use super::components::{Text, *};
use crate::plugins::text_converter::letter::Decorated;
use bevy::prelude::*;

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct Dot;

#[derive(Bundle)]
pub struct DotBundle {
    pub name: Name,
    pub dot: Dot,
}

impl DotBundle {
    pub fn new() -> Self {
        Self {
            name: Name::new("Dot"),
            dot: Dot,
        }
    }
}

pub fn convert_dots(
    mut commands: Commands,
    mut letter_query: Query<(Entity, &Letter, &mut CircleChildren), Changed<Text>>,
    mut dot_query: Query<Entity, (With<Dot>, Without<Letter>)>,
) {
    for (letter_entity, letter, mut children) in letter_query.iter_mut() {
        let mut existing_dots = dot_query.iter_many_mut(children.iter());

        let number_of_dots = letter.dots();
        let mut new_dots_iter = 0..number_of_dots;

        let mut new_children: Vec<Entity> = Vec::with_capacity(number_of_dots);

        loop {
            let next_existing_dot = existing_dots.fetch_next();
            let next_new_dot = new_dots_iter.next();

            match (next_existing_dot, next_new_dot) {
                // update dot
                (Some(dot_entity), Some(_)) => {
                    new_children.push(dot_entity);
                }
                // remove dot
                (Some(dot_entity), None) => {
                    debug!("Despawn dot");
                    commands.entity(dot_entity).despawn_recursive();
                }
                // add dot
                (None, Some(_)) => {
                    debug!("Spawn dot");

                    let bundle = DotBundle::new();

                    let dot_entity = commands.spawn(bundle).id();
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
