use crate::plugins::svg::circle::Circle;
use crate::plugins::svg::group::Group;
use crate::plugins::svg::line::Line;
use bevy::prelude::Component;
use bevy_prototype_lyon::geometry::Geometry;
use bevy_prototype_lyon::prelude::tess::path::Builder;

#[derive(Debug, Clone, Component)]
pub enum SVGElement {
    Group(Group),
    Circle(Circle),
    Line(Line),
}

impl Default for SVGElement {
    fn default() -> Self {
        Self::Group(Group::default())
    }
}

impl Geometry for SVGElement {
    fn add_geometry(&self, b: &mut Builder) {
        match self {
            SVGElement::Group(it) => {
                it.add_geometry(b);
            }
            SVGElement::Circle(it) => {
                it.add_geometry(b);
            }
            SVGElement::Line(it) => {
                it.add_geometry(b);
            }
        }
    }
}

impl From<Group> for SVGElement {
    fn from(value: Group) -> Self {
        Self::Group(value)
    }
}

impl From<Circle> for SVGElement {
    fn from(value: Circle) -> Self {
        Self::Circle(value)
    }
}

impl From<Line> for SVGElement {
    fn from(value: Line) -> Self {
        Self::Line(value)
    }
}
