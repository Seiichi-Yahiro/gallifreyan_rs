use super::{Class, Indent};
use bevy::prelude::Vec2;
use bevy_prototype_lyon::prelude::tess::path::path::Builder;
use bevy_prototype_lyon::prelude::Geometry;
use bevy_prototype_lyon::shapes;
use itertools::Itertools;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone)]
pub struct Line {
    pub from: Vec2,
    pub to: Vec2,
    pub class: Class,
}

impl Line {
    pub fn new(from: Vec2, to: Vec2) -> Self {
        Self {
            from,
            to,
            class: Class::default(),
        }
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let attributes = [
            format!("x1=\"{}\"", self.from.x),
            format!("y1=\"{}\"", self.from.y),
            format!("x2=\"{}\"", self.to.x),
            format!("y2=\"{}\"", self.to.y),
            format!("{}", self.class),
        ];

        write!(
            f,
            "<line {}/>",
            attributes.into_iter().filter(|it| !it.is_empty()).join(" ")
        )
    }
}

impl Geometry for Line {
    fn add_geometry(&self, b: &mut Builder) {
        shapes::Line(self.from, self.to).add_geometry(b);
    }
}

impl Indent for Line {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_line_tag_without_class() {
        let line = Line::new(Vec2::X, Vec2::Y);
        let result = format!("{}", line);

        let expected = r#"<line x1="1" y1="0" x2="0" y2="1"/>"#;

        assert_eq!(result, expected);
    }

    #[test]
    fn should_create_line_tag_with_class() {
        let mut line = Line::new(Vec2::X, Vec2::Y);
        line.class = Class("foo".to_string());
        let result = format!("{}", line);

        let expected = r#"<line x1="1" y1="0" x2="0" y2="1" class="foo"/>"#;

        assert_eq!(result, expected);
    }
}
