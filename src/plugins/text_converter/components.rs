use crate::math::Degree;
use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct GFText(pub String);

#[derive(Debug, Default, Component)]
pub struct CircleChildren(pub Vec<Entity>);

#[derive(Debug, Default, Component)]
pub struct LineSlotChildren(pub Vec<Entity>);

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Component)]
pub struct SiblingIndex(pub usize);

#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd, Component)]
pub struct Radius(pub f32);

#[derive(Debug, Default, Copy, Clone, PartialEq, Component)]
pub struct PositionData {
    pub angle: Degree,
    pub distance: f32,
}
