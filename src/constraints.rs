use crate::image_types::{
    CircleChildren, Dot, Letter, LineSlot, LineSlotChildren, PositionData, Radius, Sentence, Word,
};
use crate::math::Angle;
use bevy::prelude::*;

pub struct ConstraintsPlugin;

impl Plugin for ConstraintsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_word_distance_constraints)
            .add_system(update_dot_distance_constraints.after(update_word_distance_constraints))
            .add_system(
                update_line_slot_distance_constraints.after(update_word_distance_constraints),
            )
            .add_system(
                on_distance_constraints_update
                    .after(update_dot_distance_constraints)
                    .after(update_line_slot_distance_constraints),
            );
    }
}

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct AngleConstraints {
    pub min: Angle,
    pub max: Angle,
}

#[derive(Debug, Copy, Clone, Component)]
pub struct DistanceConstraints {
    pub min: f32,
    pub max: f32,
}

impl Default for DistanceConstraints {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: f32::MAX,
        }
    }
}

fn update_word_distance_constraints(
    changed_sentence_query: Query<(&Radius, &CircleChildren), (Changed<Radius>, With<Sentence>)>,
    mut word_set: ParamSet<(
        Query<(&Radius, &mut DistanceConstraints), With<Word>>,
        Query<(&Parent, &Radius, &mut DistanceConstraints), (Changed<Radius>, With<Word>)>,
    )>,
    radius_query: Query<&Radius>,
) {
    let create_constraints = |sentence_radius: &f32, word_radius: &f32| DistanceConstraints {
        min: 0.0,
        max: (sentence_radius - word_radius).max(0.0),
    };

    for (Radius(sentence_radius), words) in changed_sentence_query.iter() {
        let mut word_query = word_set.p0();
        let mut word_iter = word_query.iter_many_mut(words.iter());

        while let Some((Radius(word_radius), mut distance_constraints)) = word_iter.fetch_next() {
            *distance_constraints = create_constraints(sentence_radius, word_radius);
        }
    }

    let mut changed_word_query = word_set.p1();
    for (sentence, Radius(word_radius), mut distance_constraints) in changed_word_query.iter_mut() {
        if let Ok(Radius(sentence_radius)) = radius_query.get(sentence.get()) {
            *distance_constraints = create_constraints(sentence_radius, word_radius);
        }
    }
}

fn update_dot_distance_constraints(
    changed_radius_query: Query<(&Letter, &Radius, &CircleChildren), Changed<Radius>>,
    mut dot_query: Query<&mut DistanceConstraints, With<Dot>>,
) {
    for (letter, radius, dots) in changed_radius_query.iter() {
        match letter {
            Letter::Vocal(_) => {
                continue;
            }
            Letter::Consonant(_) => {
                let mut dot_iter = dot_query.iter_many_mut(dots.iter());

                while let Some(mut distance_constraints) = dot_iter.fetch_next() {
                    *distance_constraints = DistanceConstraints {
                        min: 0.0,
                        max: **radius,
                    }
                }
            }
        }
    }
}

fn update_line_slot_distance_constraints(
    changed_radius_query: Query<(&Radius, &LineSlotChildren), Changed<Radius>>,
    mut line_slot_query: Query<&mut DistanceConstraints, With<LineSlot>>,
) {
    for (radius, line_slots) in changed_radius_query.iter() {
        let mut line_slot_iter = line_slot_query.iter_many_mut(line_slots.iter());

        while let Some(mut distance_constraints) = line_slot_iter.fetch_next() {
            *distance_constraints = DistanceConstraints {
                min: **radius,
                max: **radius,
            }
        }
    }
}

fn on_distance_constraints_update(
    mut changed_distance_constraints_query: Query<
        (&mut PositionData, &DistanceConstraints),
        Changed<DistanceConstraints>,
    >,
) {
    for (mut position_data, distance_constraints) in changed_distance_constraints_query.iter_mut() {
        let new_distance = position_data
            .distance
            .clamp(distance_constraints.min, distance_constraints.max);

        if position_data.distance != new_distance {
            position_data.distance = new_distance;
        }
    }
}
