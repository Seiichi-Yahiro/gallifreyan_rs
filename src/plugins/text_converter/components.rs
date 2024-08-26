use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct GFText(pub String);

#[derive(Debug, Default, Component)]
pub struct CircleChildren(pub Vec<Entity>);

#[derive(Debug, Default, Component)]
pub struct LineSlotChildren(pub Vec<Entity>);

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Component)]
pub struct SiblingIndex(pub usize);
