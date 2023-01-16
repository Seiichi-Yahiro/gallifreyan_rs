use super::{Indent, SVGElement, ToCSSString, DEFAULT_INDENTATION_DEPTH};
use crate::svg::Class;
use bevy::math::Affine2;
use bevy::prelude::{FromReflect, Reflect};
use bevy_prototype_lyon::prelude::tess::path::path::Builder;
use bevy_prototype_lyon::prelude::Geometry;
use itertools::Itertools;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone, Reflect, FromReflect)]
pub struct Group {
    pub elements: Vec<SVGElement>,
    pub affine2: Affine2,
    pub class: Class,
}

impl Group {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            affine2: Affine2::IDENTITY,
            class: Class::default(),
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
            class: Class::default(),
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
            "<g transform=\"{}\" {}>\n{}\n</g>",
            self.affine2.to_css_string(),
            self.class,
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

impl Indent for Group {}
