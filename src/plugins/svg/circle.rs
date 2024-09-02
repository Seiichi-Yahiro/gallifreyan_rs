use bevy_prototype_lyon::prelude::tess::path::Builder;
use bevy_prototype_lyon::prelude::Geometry;
use bevy_prototype_lyon::shapes;

#[derive(Debug, Default, Clone)]
pub struct Circle {
    pub radius: f32,
}

impl Circle {
    pub fn new(radius: f32) -> Self {
        Self { radius }
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
