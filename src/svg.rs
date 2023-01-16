mod circle;
mod group;
mod line;
mod path;
mod style;
mod title;

pub use circle::*;
pub use group::*;
pub use line::*;
pub use path::*;
pub use style::*;
pub use title::*;

use bevy::log::error;
use bevy::math::{Affine2, Mat2};
use bevy::prelude::{Component, FromReflect, Reflect, ReflectComponent, Transform};
use bevy_prototype_lyon::geometry::Geometry;
use bevy_prototype_lyon::prelude::tess::path::path::Builder;
use itertools::Itertools;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Add;

const DEFAULT_INDENTATION_DEPTH: usize = 4;

pub trait Indent: Display {
    fn indent(&self, depth: usize) -> String {
        let indentation = " ".repeat(depth);
        self.to_string()
            .lines()
            .map(|line| indentation.clone() + line)
            .join("\n")
    }
}

impl Indent for SVG {}
impl Indent for SVGElement {}

pub struct SVG {
    pub size: f32,
    pub elements: Vec<SVGElement>,
}

impl SVG {
    pub fn new(size: f32) -> Self {
        Self {
            size,
            elements: Vec::new(),
        }
    }

    pub fn push(&mut self, element: impl Into<SVGElement>) {
        self.elements.push(element.into());
    }
}

impl Display for SVG {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let document_declaration = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>";

        let header = [
            "<svg",
            "xmlns=\"http://www.w3.org/2000/svg\"",
            "xmlns:xlink=\"http://www.w3.org/1999/xlink\"",
            &format!(
                "viewBox=\"{} {} {} {}\"",
                -self.size / 2.0,
                -self.size / 2.0,
                self.size,
                self.size
            ),
        ]
        .join("\n  ")
        .add("\n>");

        let content = self
            .elements
            .iter()
            .map(|element| element.indent(DEFAULT_INDENTATION_DEPTH))
            .join("\n");

        let footer = "</svg>";

        write!(
            f,
            "{}\n{}\n{}\n{}",
            document_declaration, header, content, footer
        )
    }
}

#[derive(Debug, Clone, Component, Reflect, FromReflect)]
#[reflect(Component)]
pub enum SVGElement {
    Title(Title),
    Group(Group),
    Circle(Circle),
    Line(Line),
    Path(Path),
    Style(Style),
}

impl Default for SVGElement {
    fn default() -> Self {
        Self::Group(Group::default())
    }
}

impl Geometry for SVGElement {
    fn add_geometry(&self, b: &mut Builder) {
        match self {
            SVGElement::Title(_) => {
                error!("Cannot convert title to geometry!");
            }
            SVGElement::Group(it) => {
                it.add_geometry(b);
            }
            SVGElement::Circle(it) => {
                it.add_geometry(b);
            }
            SVGElement::Line(it) => {
                it.add_geometry(b);
            }
            SVGElement::Path(it) => {
                it.add_geometry(b);
            }
            SVGElement::Style(_) => {
                error!("Cannot convert style to geometry!");
            }
        }
    }
}

impl From<Title> for SVGElement {
    fn from(value: Title) -> Self {
        Self::Title(value)
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

impl From<Path> for SVGElement {
    fn from(value: Path) -> Self {
        Self::Path(value)
    }
}

impl From<Style> for SVGElement {
    fn from(value: Style) -> Self {
        Self::Style(value)
    }
}

impl Display for SVGElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SVGElement::Title(it) => Display::fmt(it, f),
            SVGElement::Group(it) => Display::fmt(it, f),
            SVGElement::Circle(it) => Display::fmt(it, f),
            SVGElement::Line(it) => Display::fmt(it, f),
            SVGElement::Path(it) => Display::fmt(it, f),
            SVGElement::Style(it) => Display::fmt(it, f),
        }
    }
}

pub trait ToAffine2 {
    fn to_affine2(&self) -> Affine2;
}

impl ToAffine2 for Transform {
    fn to_affine2(&self) -> Affine2 {
        let affine3 = self.compute_affine();

        Affine2 {
            matrix2: Mat2::from_cols(affine3.x_axis.truncate(), affine3.y_axis.truncate()),
            translation: affine3.translation.truncate(),
        }
    }
}

pub trait ToCSSString {
    fn to_css_string(&self) -> String;
}

impl ToCSSString for Affine2 {
    fn to_css_string(&self) -> String {
        let values = self.to_cols_array().map(|it| it.to_string()).join(" ");

        format!("matrix({})", values)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bevy::prelude::Vec2;

    #[test]
    fn should_create_simple_svg() {
        let title = Title("TITLE".to_string());

        let circle = Circle { radius: 1.0 };

        let line = Line {
            from: Vec2::ZERO,
            to: Vec2::ONE,
        };

        let path = Path(vec![
            PathElement::MoveTo(Vec2::ZERO),
            PathElement::Arc {
                radius: 10.0,
                large_arc: false,
                end: Vec2::ONE,
            },
        ]);

        let group1 = Group {
            elements: vec![circle.into(), line.into()],
            affine2: Default::default(),
        };

        let group2 = Group {
            elements: vec![group1.into(), path.into()],
            affine2: Default::default(),
        };

        let svg = SVG {
            size: 100.0,
            elements: vec![title.into(), group2.into()],
        };

        let result = svg.to_string();

        let expected = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg
  xmlns="http://www.w3.org/2000/svg"
  xmlns:xlink="http://www.w3.org/1999/xlink"
  viewBox="-50 -50 100 100"
>
    <title>TITLE</title>
    <g transform="matrix(1 0 0 1 0 0)">
        <g transform="matrix(1 0 0 1 0 0)">
            <circle cx="0" cy="0" r="1" />
            <line x1="0" y1="0" x2="1" y2="1"/>
        </g>
        <path d="M 0 0 A 10 10 0 0 1 1 1" />
    </g>
</svg>"#;

        assert_eq!(result, expected);
    }
}
