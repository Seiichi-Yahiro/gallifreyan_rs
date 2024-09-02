use bevy::math::Vec2;
use bevy_prototype_lyon::prelude::tess::path::Builder;
use bevy_prototype_lyon::prelude::Geometry;
use bevy_prototype_lyon::shapes;

#[derive(Debug, Default, Clone)]
pub struct Line {
    pub from: Vec2,
    pub to: Vec2,
}

impl Line {
    pub fn new(from: Vec2, to: Vec2) -> Self {
        Self { from, to }
    }
}

impl Geometry for Line {
    fn add_geometry(&self, b: &mut Builder) {
        shapes::Line(self.from, self.to).add_geometry(b);
    }
}
