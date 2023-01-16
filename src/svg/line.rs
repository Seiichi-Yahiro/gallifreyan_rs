use bevy::prelude::{FromReflect, Reflect, Vec2};
use bevy_prototype_lyon::prelude::tess::path::path::Builder;
use bevy_prototype_lyon::prelude::Geometry;
use bevy_prototype_lyon::shapes;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Copy, Clone, Reflect, FromReflect)]
pub struct Line {
    pub from: Vec2,
    pub to: Vec2,
}

impl Display for Line {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"/>",
            self.from.x, self.from.y, self.to.x, self.to.y
        )
    }
}

impl Geometry for Line {
    fn add_geometry(&self, b: &mut Builder) {
        shapes::Line(self.from, self.to).add_geometry(b);
    }
}
