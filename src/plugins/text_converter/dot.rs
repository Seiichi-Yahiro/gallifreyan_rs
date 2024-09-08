use crate::math::angle::Degree;
use crate::plugins::text_converter::prelude::*;
use bevy::prelude::*;

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct Dot;

#[derive(Bundle)]
pub struct DotBundle {
    pub name: Name,
    pub dot: Dot,
    pub sibling_index: SiblingIndex,
    pub radius: Radius,
    pub position_data: PositionData,
}

impl DotBundle {
    pub fn new(sibling_index: usize) -> Self {
        Self {
            name: Name::new("Dot"),
            dot: Dot,
            sibling_index: SiblingIndex(sibling_index),
            radius: Radius::default(),
            position_data: PositionData::default(),
        }
    }
}

pub fn convert_dots(
    mut commands: Commands,
    mut letter_query: Query<(Entity, &Letter, &mut CircleChildren), Changed<GFText>>,
    mut dot_query: Query<(Entity, &mut SiblingIndex), (With<Dot>, Without<Letter>)>,
) {
    for (letter_entity, letter, mut children) in letter_query.iter_mut() {
        let mut existing_dots = dot_query.iter_many_mut(children.0.iter());

        let number_of_dots = letter.dots();
        let mut new_dots_iter = 0..number_of_dots;

        let mut new_children: Vec<Entity> = Vec::with_capacity(number_of_dots);

        loop {
            let next_existing_dot = existing_dots.fetch_next();
            let next_new_dot = new_dots_iter.next();
            let sibling_index = new_children.len();

            match (next_existing_dot, next_new_dot) {
                // update dot
                (Some((dot_entity, mut dot_sibling_index)), Some(_)) => {
                    dot_sibling_index.0 = sibling_index;
                    new_children.push(dot_entity);
                }
                // remove dot
                (Some((dot_entity, _dot_sibling_index)), None) => {
                    debug!("Despawn dot: {:?}", dot_entity);
                    commands.entity(dot_entity).despawn_recursive();
                }
                // add dot
                (None, Some(_)) => {
                    let bundle = DotBundle::new(sibling_index);

                    let dot_entity = commands.spawn(bundle).id();
                    debug!("Spawn dot: {:?}", dot_entity);

                    commands.entity(letter_entity).add_child(dot_entity);
                    new_children.push(dot_entity);
                }
                (None, None) => {
                    break;
                }
            }
        }

        children.0 = new_children;
    }
}

pub fn set_default_radius(
    letter_query: Query<&Radius, (With<Letter>, Without<Dot>)>,
    mut dot_query: Query<(&Parent, &mut Radius), (With<Dot>, Without<Letter>)>,
) {
    for (dot_parent, mut dot_radius) in dot_query.iter_mut() {
        let letter_radius = letter_query.get(dot_parent.get()).unwrap();

        dot_radius.0 = letter_radius.0 * 0.1;
    }
}

pub fn set_default_position(
    letter_query: Query<(&Radius, &CircleChildren), (With<Letter>, Without<Dot>)>,
    mut dot_query: Query<
        (&Parent, &mut PositionData, &Radius, &SiblingIndex),
        (With<Dot>, Without<Letter>),
    >,
) {
    for (dot_parent, mut position_data, dot_radius, dot_index) in dot_query.iter_mut() {
        let (letter_radius, letter_children) = letter_query.get(dot_parent.get()).unwrap();

        let number_of_dots = letter_children.0.len();

        const LETTER_SIDE_ANGLE: f32 = 180.0;
        const DOT_DISTANCE_ANGLE: f32 = 45.0;

        let center_dots_on_letter_side_angle: f32 =
            ((number_of_dots - 1) as f32 * DOT_DISTANCE_ANGLE) / 2.0;

        position_data.distance = letter_radius.0 - dot_radius.0 * 1.5;

        position_data.angle = Degree(
            dot_index.0 as f32 * DOT_DISTANCE_ANGLE - center_dots_on_letter_side_angle
                + LETTER_SIDE_ANGLE,
        );
    }
}
