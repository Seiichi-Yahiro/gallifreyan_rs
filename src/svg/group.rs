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
        let attributes = [
            format!("transform=\"{}\"", self.affine2.to_css_string()),
            format!("{}", self.class),
        ];

        let content = self
            .elements
            .iter()
            .map(|element| element.indent(DEFAULT_INDENTATION_DEPTH))
            .join("\n");

        write!(
            f,
            "<g {}>\n{}\n</g>",
            attributes.into_iter().filter(|it| !it.is_empty()).join(" "),
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_nested_group_tag_without_class() {
        let mut group = Group::new();
        group.push(Group::new());

        let result = format!("{}", group);

        let expected = r#"<g transform="matrix(1 0 0 1 0 0)">
    <g transform="matrix(1 0 0 1 0 0)">
    
    </g>
</g>"#;

        assert_eq!(result, expected);
    }

    #[test]
    fn should_create_group_tag_with_class() {
        let mut group = Group::new();
        group.class = Class("foo".to_string());
        group.push(Group::new());

        let result = format!("{}", group);

        let expected = r#"<g transform="matrix(1 0 0 1 0 0)" class="foo">
    <g transform="matrix(1 0 0 1 0 0)">
    
    </g>
</g>"#;

        assert_eq!(result, expected);
    }
}
