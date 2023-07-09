use super::{Class, Indent};
use bevy_prototype_lyon::prelude::tess::path::path::Builder;
use bevy_prototype_lyon::prelude::Geometry;
use bevy_prototype_lyon::shapes;
use itertools::Itertools;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone)]
pub struct Circle {
    pub radius: f32,
    pub class: Class,
}

impl Circle {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            class: Class::default(),
        }
    }
}

impl Display for Circle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let attributes = [
            "cx=\"0\"".to_string(),
            "cy=\"0\"".to_string(),
            format!("r=\"{}\"", self.radius),
            format!("{}", self.class),
        ];

        write!(
            f,
            "<circle {}/>",
            attributes.into_iter().filter(|it| !it.is_empty()).join(" ")
        )
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

impl Indent for Circle {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_circle_tag_without_class() {
        let circle = Circle::new(10.0);
        let result = format!("{}", circle);

        let expected = r#"<circle cx="0" cy="0" r="10"/>"#;

        assert_eq!(result, expected);
    }

    #[test]
    fn should_create_circle_tag_with_class() {
        let mut circle = Circle::new(10.0);
        circle.class = Class("foo".to_string());
        let result = format!("{}", circle);

        let expected = r#"<circle cx="0" cy="0" r="10" class="foo"/>"#;

        assert_eq!(result, expected);
    }
}
