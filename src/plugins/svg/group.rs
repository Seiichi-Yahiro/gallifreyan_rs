use crate::plugins::svg::SVGElement;
use bevy::math::Affine2;
use bevy_prototype_lyon::prelude::tess::path::Builder;
use bevy_prototype_lyon::prelude::Geometry;

#[derive(Debug, Default, Clone)]
pub struct Group {
    pub elements: Vec<SVGElement>,
    pub affine2: Affine2,
}

impl Group {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            affine2: Affine2::IDENTITY,
        }
    }

    pub fn push(&mut self, element: impl Into<SVGElement>) {
        self.elements.push(element.into());
    }
}

impl From<Vec<SVGElement>> for Group {
    fn from(value: Vec<SVGElement>) -> Self {
        Self {
            elements: value,
            affine2: Affine2::IDENTITY,
        }
    }
}

impl Geometry for Group {
    fn add_geometry(&self, b: &mut Builder) {
        for element in &self.elements {
            element.add_geometry(b);
        }
    }
}
