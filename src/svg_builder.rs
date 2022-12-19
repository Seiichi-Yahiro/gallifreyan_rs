use bevy::math::Mat3;
use bevy::prelude::Transform;
use itertools::Itertools;
use std::fmt::{Display, Formatter};
use std::ops::Add;

pub trait SvgItem: Display {
    fn build(&self, indentation_level: usize) -> String;
}

pub struct SVGBuilder {
    size: f32,
    content: Vec<Box<dyn SvgItem + 'static>>,
}

impl SVGBuilder {
    pub fn new(size: f32) -> Self {
        Self {
            size,
            content: Vec::new(),
        }
    }

    pub fn add<T: SvgItem + 'static>(&mut self, svg: T) -> &mut Self {
        self.content.push(Box::new(svg));
        self
    }

    pub fn build(&self) -> String {
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

        let content = self.content.iter().map(|it| it.build(1)).join("\n");

        let footer = "</svg>";

        format!(
            "{}\n{}\n{}\n{}",
            document_declaration, header, content, footer
        )
    }
}

impl Display for SVGBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build())
    }
}

pub struct GroupBuilder {
    transform: Mat3,
    content: Vec<Box<dyn SvgItem>>,
}

impl GroupBuilder {
    pub fn new() -> Self {
        Self {
            transform: Mat3::IDENTITY,
            content: Vec::new(),
        }
    }

    pub fn with_transform(mut self, transform: Mat3) -> Self {
        self.transform = transform;
        self
    }

    pub fn add<T: SvgItem + 'static>(&mut self, svg: T) -> &mut Self {
        self.content.push(Box::new(svg));
        self
    }
}

impl SvgItem for GroupBuilder {
    fn build(&self, indentation_level: usize) -> String {
        let indentation = "    ".repeat(indentation_level);

        let content = self
            .content
            .iter()
            .map(|it| it.build(indentation_level + 1))
            .join("\n");

        format!(
            "{}<g transform=\"{}\">\n{}\n{}</g>",
            indentation,
            mat3_to_string(self.transform),
            content,
            indentation
        )
    }
}

impl Display for GroupBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build(0))
    }
}

#[derive(Copy, Clone)]
pub enum Fill {
    Black,
    None,
}

impl Display for Fill {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let it = match self {
            Fill::Black => "black",
            Fill::None => "none",
        };

        write!(f, "{}", it)
    }
}

#[derive(Copy, Clone)]
pub enum Stroke {
    Black,
    White,
}

impl Display for Stroke {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let it = match self {
            Stroke::Black => "black",
            Stroke::White => "white",
        };

        write!(f, "{}", it)
    }
}

pub struct CircleBuilder {
    radius: f32,
    transform: Mat3,
    stroke: Stroke,
    fill: Fill,
    mask: Option<String>,
}

impl SvgItem for CircleBuilder {
    fn build(&self, indentation_level: usize) -> String {
        let indentation = "    ".repeat(indentation_level);

        let circle = [
            format!("{}<circle", indentation),
            format!("cx=\"0\" cy=\"0\" r=\"{}\"", self.radius),
            format!("stroke=\"{}\" fill=\"{}\"", self.stroke, self.fill),
            format!("transform=\"{}\"", mat3_to_string(self.transform)),
        ];

        let mask = self
            .mask
            .as_ref()
            .map(|id| format!("mask=\"url(#{})\"", id));

        circle
            .into_iter()
            .chain(mask)
            .join(&format!("\n  {}", indentation))
            .add(&format!("\n{}/>", indentation))
    }
}

impl Display for CircleBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build(0))
    }
}

impl CircleBuilder {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            transform: Mat3::IDENTITY,
            stroke: Stroke::Black,
            fill: Fill::None,
            mask: None,
        }
    }

    pub fn with_transform(mut self, transform: Mat3) -> Self {
        self.transform = transform;
        self
    }

    pub fn with_stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }

    pub fn with_fill(mut self, fill: Fill) -> Self {
        self.fill = fill;
        self
    }

    pub fn with_mask(mut self, mask: Option<String>) -> Self {
        self.mask = mask;
        self
    }
}

pub struct MaskBuilder {
    id: String,
    content: Vec<Box<dyn SvgItem>>,
}

impl MaskBuilder {
    pub fn new(id: String) -> Self {
        Self {
            id,
            content: Vec::new(),
        }
    }

    pub fn add<T: SvgItem + 'static>(&mut self, svg: T) -> &mut Self {
        self.content.push(Box::new(svg));
        self
    }
}

impl SvgItem for MaskBuilder {
    fn build(&self, indentation_level: usize) -> String {
        let indentation = "    ".repeat(indentation_level);

        let content = self
            .content
            .iter()
            .map(|it| it.build(indentation_level + 1))
            .join("\n");

        format!(
            "{}<mask id=\"{}\">\n{}\n{}</mask>",
            indentation, self.id, content, indentation
        )
    }
}

impl Display for MaskBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build(0))
    }
}

pub struct Title {
    content: String,
}

impl Title {
    pub fn new(content: String) -> Self {
        Self { content }
    }
}

impl SvgItem for Title {
    fn build(&self, indentation_level: usize) -> String {
        let indentation = "    ".repeat(indentation_level);

        format!("{}<title>{}</title>", indentation, self.content)
    }
}

impl Display for Title {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build(0))
    }
}

fn mat3_to_string(mat3: Mat3) -> String {
    format!(
        "matrix({} {} {} {} {} {})",
        mat3.x_axis.x, mat3.x_axis.y, mat3.y_axis.x, mat3.y_axis.y, mat3.z_axis.x, mat3.z_axis.y
    )
}

pub trait AsMat3 {
    fn as_mat3(&self, inverse: bool) -> Mat3;
}

impl AsMat3 for Transform {
    fn as_mat3(&self, inverse: bool) -> Mat3 {
        use bevy::math::swizzles::Vec4Swizzles;
        let mat4 = if inverse {
            self.compute_matrix().inverse()
        } else {
            self.compute_matrix()
        };
        Mat3::from_cols(mat4.x_axis.xyz(), mat4.y_axis.xyz(), mat4.w_axis.xyz())
    }
}
