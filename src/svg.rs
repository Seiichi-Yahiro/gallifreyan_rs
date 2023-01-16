use bevy::log::error;
use bevy::math::{Affine2, Mat2, Vec2};
use bevy::prelude::{Color, Component, FromReflect, Reflect, ReflectComponent, Transform};
use bevy_prototype_lyon::geometry::Geometry;
use bevy_prototype_lyon::prelude::tess::path::path::Builder;
use bevy_prototype_lyon::shapes;
use itertools::Itertools;
use std::fmt::{Display, Formatter};
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
impl Indent for Title {}
impl Indent for Group {}
impl Indent for Circle {}
impl Indent for Line {}
impl Indent for Path {}

pub struct SVG {
    pub size: f32,
    pub elements: Vec<SVGElement>,
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

impl Display for SVGElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SVGElement::Title(it) => it.fmt(f),
            SVGElement::Group(it) => it.fmt(f),
            SVGElement::Circle(it) => it.fmt(f),
            SVGElement::Line(it) => it.fmt(f),
            SVGElement::Path(it) => it.fmt(f),
        }
    }
}

#[derive(Debug, Default, Clone, Reflect, FromReflect)]
pub struct Title {
    text: String,
}

impl Display for Title {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<title>{}</title>", self.text)
    }
}

#[derive(Debug, Default, Clone, Reflect, FromReflect)]
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

impl Display for Group {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let content = self
            .elements
            .iter()
            .map(|element| element.indent(DEFAULT_INDENTATION_DEPTH))
            .join("\n");

        write!(
            f,
            "<g transform=\"{}\">\n{}\n</g>",
            self.affine2.to_css_string(),
            content
        )
    }
}

impl Geometry for Group {
    fn add_geometry(&self, b: &mut Builder) {
        for element in &self.elements {
            element.add_geometry(b);
        }
    }
}

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

#[derive(Debug, Copy, Clone, Reflect, FromReflect)]
pub enum DrawMode {
    Fill(Color),
    Stroke(Color),
}

impl Default for DrawMode {
    fn default() -> Self {
        Self::Stroke(Color::BLACK)
    }
}

/*impl Display for DrawMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DrawMode::Fill(color) => {
                write!(
                    f,
                    "stroke=\"none\" fill=\"rgb({}, {}, {})\"",
                    color.r() * 255.0,
                    color.g() * 255.0,
                    color.b() * 255.0
                )
            }
            DrawMode::Stroke(color) => {
                write!(
                    f,
                    "stroke=\"rgb({}, {}, {})\" fill=\"none\"",
                    color.r() * 255.0,
                    color.g() * 255.0,
                    color.b() * 255.0
                )
            }
        }
    }
}*/

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

    #[test]
    fn should_create_simple_svg() {
        let title = Title {
            text: "TITLE".to_string(),
        };

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
