use crate::image_types::{CircleChildren, PositionData, Sentence, Word};
use crate::math::angle::{Angle, Degree};
use crate::update_if_changed::update_if_changed;
use bevy::prelude::*;
use itertools::Itertools;

const MIN_ANGLE: Degree = Degree::new(0.0);
const MAX_ANGLE: Degree = Degree::new(360.0);

pub struct AngleConstraintsPlugin;

impl Plugin for AngleConstraintsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_word_angle_constraints)
            .add_system(on_angle_constraints_update.after(update_word_angle_constraints));
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub struct AngleConstraints {
    pub min: Degree,
    pub max: Degree,
}

impl Default for AngleConstraints {
    fn default() -> Self {
        Self {
            min: MIN_ANGLE,
            max: MAX_ANGLE,
        }
    }
}

fn update_word_angle_constraints(
    changed_word_query: Query<(Entity, &Parent), (Changed<PositionData>, With<Word>)>,
    sentence_query: Query<&CircleChildren, With<Sentence>>,
    mut word_query: Query<(&PositionData, &mut AngleConstraints), With<Word>>,
) {
    for (changed_word_entity, parent) in changed_word_query.iter() {
        if let Ok(words) = sentence_query.get(parent.get()) {
            let (index, _) = words
                .iter()
                .find_position(|word| **word == changed_word_entity)
                .unwrap();

            let left = index.checked_sub(1).and_then(|i| words.get(i));
            let right = index.checked_add(1).and_then(|i| words.get(i));

            match (left, right) {
                (Some(left), Some(right)) => {
                    if let Ok([left, middle, right]) =
                        word_query.get_many_mut([*left, changed_word_entity, *right])
                    {
                        let (left_position_data, mut left_angle_constraints) = left;
                        let (middle_position_data, mut middle_angle_constraints) = middle;
                        let (right_position_data, mut right_angle_constraints) = right;

                        let new_middle_angle_constraints = AngleConstraints {
                            min: left_position_data.angle,
                            max: right_position_data.angle,
                        };

                        update_if_changed!(*middle_angle_constraints, new_middle_angle_constraints);
                        left_angle_constraints.max = middle_position_data.angle;
                        right_angle_constraints.min = middle_position_data.angle;
                    }
                }
                (Some(left), None) => {
                    if let Ok([left, middle]) =
                        word_query.get_many_mut([*left, changed_word_entity])
                    {
                        let (left_position_data, mut left_angle_constraints) = left;
                        let (middle_position_data, mut middle_angle_constraints) = middle;

                        let new_middle_angle_constraints = AngleConstraints {
                            min: left_position_data.angle,
                            max: MAX_ANGLE,
                        };

                        update_if_changed!(*middle_angle_constraints, new_middle_angle_constraints);
                        left_angle_constraints.max = middle_position_data.angle;
                    }
                }
                (None, Some(right)) => {
                    if let Ok([middle, right]) =
                        word_query.get_many_mut([changed_word_entity, *right])
                    {
                        let (middle_position_data, mut middle_angle_constraints) = middle;
                        let (right_position_data, mut right_angle_constraints) = right;

                        let new_middle_angle_constraints = AngleConstraints {
                            min: MIN_ANGLE,
                            max: right_position_data.angle,
                        };

                        update_if_changed!(*middle_angle_constraints, new_middle_angle_constraints);
                        right_angle_constraints.min = middle_position_data.angle;
                    }
                }
                (None, None) => {
                    if let Ok((_, mut angle_constraints)) = word_query.get_mut(changed_word_entity)
                    {
                        let new_angle_constraints = AngleConstraints::default();
                        update_if_changed!(*angle_constraints, new_angle_constraints);
                    }
                }
            }
        }
    }
}

fn on_angle_constraints_update(
    mut changed_angle_constraints_query: Query<
        (&mut PositionData, &AngleConstraints),
        Changed<AngleConstraints>,
    >,
) {
    for (mut position_data, angle_constraints) in changed_angle_constraints_query.iter_mut() {
        let new_angle = position_data
            .angle
            .normalize()
            .clamp(angle_constraints.min, angle_constraints.max);

        update_if_changed!(position_data.angle, new_angle, "Update angle: {:?} -> {:?}");
    }
}
