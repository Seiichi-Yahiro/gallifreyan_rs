use super::Indent;
use crate::svg::Class;
use bevy::prelude::{FromReflect, Reflect, Vec2};
use bevy_prototype_lyon::prelude::tess::path::path::Builder;
use bevy_prototype_lyon::prelude::Geometry;
use bevy_prototype_lyon::shapes;
use itertools::Itertools;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone, Reflect, FromReflect)]
pub struct Path {
    pub elements: Vec<PathElement>,
    pub class: Class,
}

impl Path {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            class: Class::default(),
        }
    }

    pub fn push(&mut self, element: PathElement) {
        self.elements.push(element);
    }

    pub fn path(&self) -> String {
        self.elements.iter().map(ToString::to_string).join(" ")
    }
}

impl From<Vec<PathElement>> for Path {
    fn from(value: Vec<PathElement>) -> Self {
        Self {
            elements: value,
            class: Class::default(),
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<path d=\"{}\" {}/>", self.path(), self.class)
    }
}

impl Geometry for Path {
    fn add_geometry(&self, b: &mut Builder) {
        shapes::SvgPathShape {
            svg_doc_size_in_px: Default::default(),
            svg_path_string: self.path(),
        }
        .add_geometry(b);
    }
}

#[derive(Debug, Copy, Clone, Reflect, FromReflect)]
pub enum PathElement {
    MoveTo(Vec2),
    Arc {
        radius: f32,
        large_arc: bool,
        end: Vec2,
    },
}

impl Display for PathElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PathElement::MoveTo(pos) => {
                write!(f, "M {} {}", pos.x, pos.y)
            }
            PathElement::Arc {
                radius,
                large_arc,
                end,
            } => {
                write!(
                    f,
                    "A {} {} 0 {} 1 {} {}",
                    radius,
                    radius,
                    i32::from(*large_arc),
                    end.x,
                    end.y
                )
            }
        }
    }
}

impl Indent for Path {}
