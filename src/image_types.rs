mod dot;
mod letter;
mod line_slot;
mod sentence;
mod word;

pub use dot::*;
pub use letter::*;
pub use line_slot::*;
pub use sentence::*;
pub use word::*;

use crate::math::Angle;
use crate::style::DEFAULT_SVG_COLOR;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{
    FillMode, FillOptions, LineCap, LineJoin, StrokeMode, StrokeOptions,
};

pub const SVG_SIZE: f32 = 1000.0;
const STROKE_OPTIONS: StrokeOptions = StrokeOptions::DEFAULT
    .with_line_cap(LineCap::Round)
    .with_line_join(LineJoin::Round)
    .with_line_width(1.0);

const STROKE_MODE: StrokeMode = StrokeMode {
    options: STROKE_OPTIONS,
    color: DEFAULT_SVG_COLOR,
};

const FILL_MODE: FillMode = FillMode {
    options: FillOptions::DEFAULT,
    color: DEFAULT_SVG_COLOR,
};

#[derive(Default, Component, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct CircleChildren(pub Vec<Entity>);

#[derive(Default, Component, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct LineSlotChildren(pub Vec<Entity>);

#[derive(Debug, Default, PartialEq, Eq, Component, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct Text(pub String);

#[derive(Debug, Default, Copy, Clone, PartialEq, Component, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct Radius(pub f32);

#[derive(Debug, Default, Copy, Clone, PartialEq, Component, Reflect)]
#[reflect(Component)]
pub struct PositionData {
    pub angle: Angle,
    pub distance: f32,
    pub angle_placement: AnglePlacement,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Reflect)]
pub enum AnglePlacement {
    Absolute,
    Relative,
}

impl Default for AnglePlacement {
    fn default() -> Self {
        Self::Absolute
    }
}
