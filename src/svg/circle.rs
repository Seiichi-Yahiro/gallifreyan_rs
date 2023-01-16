use bevy::prelude::{FromReflect, Reflect};
use bevy_prototype_lyon::prelude::tess::path::path::Builder;
use bevy_prototype_lyon::prelude::Geometry;
use bevy_prototype_lyon::shapes;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Copy, Clone, Reflect, FromReflect)]
pub struct Circle {
    pub radius: f32,
}

impl Circle {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl Display for Circle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<circle cx=\"0\" cy=\"0\" r=\"{}\" />", self.radius)
    }
}

impl Geometry for Circle {
    fn add_geometry(&self, b: &mut Builder) {
        shapes::Circle {
            radius: self.radius,
            center: Default::default(),
        }
        .add_geometry(b);
    }
}
