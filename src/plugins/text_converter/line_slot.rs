use crate::math::Degree;
use crate::plugins::text_converter::prelude::*;
use bevy::prelude::*;

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct LineSlot;

#[derive(Bundle)]
pub struct LineSlotBundle {
    pub name: Name,
    pub line_slot: LineSlot,
    pub sibling_index: SiblingIndex,
    pub position_data: PositionData,
}

impl LineSlotBundle {
    pub fn new(sibling_index: usize) -> Self {
        Self {
            name: Name::new("Line Slot"),
            line_slot: LineSlot,
            sibling_index: SiblingIndex(sibling_index),
            position_data: PositionData::default(),
        }
    }
}

pub fn convert_line_slots(
    mut commands: Commands,
    mut letter_query: Query<(Entity, &Letter, &mut LineSlotChildren), Changed<GFText>>,
    mut line_slot_query: Query<(Entity, &mut SiblingIndex), (With<LineSlot>, Without<Letter>)>,
) {
    for (letter_entity, letter, mut children) in letter_query.iter_mut() {
        let mut existing_line_slots = line_slot_query.iter_many_mut(children.0.iter());

        let number_of_lines = letter.lines();
        let mut new_line_slots_iter = 0..number_of_lines;

        let mut new_children: Vec<Entity> = Vec::with_capacity(number_of_lines);

        loop {
            let next_existing_line_slot = existing_line_slots.fetch_next();
            let next_new_line_slot = new_line_slots_iter.next();
            let sibling_index = new_children.len();

            match (next_existing_line_slot, next_new_line_slot) {
                // update line slot
                (Some((line_slot_entity, mut line_slot_sibling_index)), Some(_)) => {
                    line_slot_sibling_index.0 = sibling_index;
                    new_children.push(line_slot_entity);
                }
                // remove line slot
                (Some((line_slot_entity, _line_slot_sibling_index)), None) => {
                    debug!("Despawn line_slot: {:?}", line_slot_entity);
                    commands.entity(line_slot_entity).despawn_recursive();
                }
                // add line slot
                (None, Some(_)) => {
                    let bundle = LineSlotBundle::new(sibling_index);

                    let line_slot_entity = commands.spawn(bundle).id();
                    debug!("Spawn line_slot: {:?}", line_slot_entity);

                    commands.entity(letter_entity).add_child(line_slot_entity);
                    new_children.push(line_slot_entity);
                }
                (None, None) => {
                    break;
                }
            }
        }

        children.0 = new_children;
    }
}

pub fn set_default_position(
    parent_query: Query<(&Radius, Option<&Letter>, &LineSlotChildren), Without<LineSlot>>,
    mut line_slot_query: Query<(&Parent, &mut PositionData, &SiblingIndex), With<LineSlot>>,
) {
    for (line_slot_parent, mut position_data, line_slot_index) in line_slot_query.iter_mut() {
        let (parent_radius, letter, line_slot_children) =
            parent_query.get(line_slot_parent.get()).unwrap();

        let number_of_lines = line_slot_children.0.len();

        let line_points_outside = letter
            .map(|it| match it {
                Letter::Vocal(vocal) => {
                    VocalDecoration::from(*vocal) == VocalDecoration::LineOutside
                }
                Letter::Consonant(_) => false,
            })
            .unwrap_or(false);

        let side_angle = if line_points_outside { 0.0 } else { 180.0 };
        const LINE_DISTANCE_ANGLE: f32 = 45.0;
        let center_lines_on_side_angle = ((number_of_lines - 1) as f32 * LINE_DISTANCE_ANGLE) / 2.0;

        position_data.distance = parent_radius.0;

        position_data.angle = Degree(
            line_slot_index.0 as f32 * LINE_DISTANCE_ANGLE - center_lines_on_side_angle
                + side_angle,
        );
    }
}
