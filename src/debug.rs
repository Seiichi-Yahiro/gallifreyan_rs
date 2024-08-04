use bevy::prelude::*;

pub fn debug_log_observer<T: Event + std::fmt::Debug>(trigger: Trigger<T>, query: Query<&Name>) {
    let entity = trigger.entity();
    let name = query.get(entity).unwrap();
    debug!("{:?}: {}, {}", trigger.event(), name, entity);
}
