use bevy::prelude::*;

pub struct InteractionsPlugin;

impl Plugin for InteractionsPlugin {
    fn build(&self, app: &mut App) {
        app.observe(add_previous_interaction)
            .observe(remove_previous_interaction)
            .add_systems(Update, interaction_events);

        #[cfg(debug_assertions)]
        {
            use crate::debug::debug_log_observer;

            app.observe(debug_log_observer::<Pressed>)
                .observe(debug_log_observer::<Released>)
                .observe(debug_log_observer::<HoverIn>)
                .observe(debug_log_observer::<HoverOut>);
        }
    }
}

#[derive(Debug, Component, Copy, Clone, Eq, PartialEq)]
struct PreviousInteraction(Interaction);

fn add_previous_interaction(
    trigger: Trigger<OnAdd, Interaction>,
    mut commands: Commands,
    query: Query<&Interaction>,
) {
    let entity = trigger.entity();
    let previous_interaction = *query.get(entity).unwrap();
    commands
        .entity(entity)
        .insert(PreviousInteraction(previous_interaction));
}

fn remove_previous_interaction(trigger: Trigger<OnRemove, Interaction>, mut commands: Commands) {
    commands
        .entity(trigger.entity())
        .remove::<PreviousInteraction>();
}

#[derive(Debug, Event, Copy, Clone)]
pub struct Pressed;

#[derive(Debug, Event, Copy, Clone)]
pub struct Released;

#[derive(Debug, Event, Copy, Clone)]
pub struct HoverIn;

#[derive(Debug, Event, Copy, Clone)]
pub struct HoverOut;

fn interaction_events(
    mut commands: Commands,
    mut query: Query<(Entity, &Interaction, &mut PreviousInteraction), Changed<Interaction>>,
) {
    for (entity, interaction, mut previous_interaction) in query.iter_mut() {
        if *interaction == previous_interaction.0 {
            continue;
        }

        match interaction {
            Interaction::Pressed => {
                commands.trigger_targets(Pressed, entity);
                previous_interaction.0 = Interaction::Pressed;
            }
            Interaction::Hovered => {
                match previous_interaction.0 {
                    Interaction::Pressed => {
                        commands.trigger_targets(Released, entity);
                    }
                    Interaction::Hovered => {
                        unreachable!();
                    }
                    Interaction::None => {
                        commands.trigger_targets(HoverIn, entity);
                    }
                }

                previous_interaction.0 = Interaction::Hovered;
            }
            Interaction::None => {
                commands.trigger_targets(HoverOut, entity);
                previous_interaction.0 = Interaction::None;
            }
        }
    }
}
