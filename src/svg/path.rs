use bevy::prelude::{FromReflect, Reflect, Vec2};
use bevy_prototype_lyon::prelude::tess::path::path::Builder;
use bevy_prototype_lyon::prelude::Geometry;
use bevy_prototype_lyon::shapes;
use itertools::Itertools;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone, Reflect, FromReflect)]
pub struct Path(pub Vec<PathElement>);

impl Path {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, element: PathElement) {
        self.0.push(element);
    }
}

impl From<Vec<PathElement>> for Path {
    fn from(value: Vec<PathElement>) -> Self {
        Self(value)
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let path = self.0.iter().map(ToString::to_string).join(" ");

        write!(f, "<path d=\"{}\" />", path)
    }
}

impl Geometry for Path {
    fn add_geometry(&self, b: &mut Builder) {
        shapes::SvgPathShape {
            svg_doc_size_in_px: Default::default(),
            svg_path_string: self.to_string(),
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
