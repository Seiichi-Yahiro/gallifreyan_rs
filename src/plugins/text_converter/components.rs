use bevy::prelude::*;

pub use super::dot::Dot;
pub use super::letter::{consonant::Consonant, vocal::Vocal, Letter};
pub use super::line_slot::LineSlot;
pub use super::sentence::Sentence;
pub use super::word::Word;

#[derive(Debug, Default, Clone, PartialEq, Eq, Component, Deref, DerefMut)]
pub struct Text(pub String);

#[derive(Default, Component, Deref, DerefMut)]
pub struct CircleChildren(pub Vec<Entity>);

#[derive(Default, Component, Deref, DerefMut)]
pub struct LineSlotChildren(pub Vec<Entity>);
