use super::Indent;
use bevy::prelude::{FromReflect, Reflect};
use bevy_prototype_lyon::prelude::tess::path::path::Builder;
use bevy_prototype_lyon::prelude::Geometry;
use bevy_prototype_lyon::shapes;
use std::fmt::{Display, Formatter};
use crate::svg::Class;

#[derive(Debug, Default, Clone, Reflect, FromReflect)]
pub struct Circle {
    pub radius: f32,
    pub class: Class,
}

impl Circle {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            class: Class::default(),
        }
    }
}

impl Display for Circle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<circle cx=\"0\" cy=\"0\" r=\"{}\" {}/>", self.radius, self.class)
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

impl Indent for Circle {}
