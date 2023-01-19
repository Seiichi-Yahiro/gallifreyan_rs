use super::{Class, Indent};
use bevy::prelude::{FromReflect, Reflect, Vec2};
use bevy_prototype_lyon::prelude::tess::path::path::Builder;
use bevy_prototype_lyon::prelude::Geometry;
use bevy_prototype_lyon::shapes;
use itertools::Itertools;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone, Reflect, FromReflect)]
pub struct Path {
    pub elements: Vec<PathElement>,
    #[reflect(ignore)]
    pub class: Class,
}

impl Path {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            class: Class::default(),
        }
    }

    pub fn push(&mut self, element: PathElement) {
        self.elements.push(element);
    }

    pub fn path(&self, flip_y_axis: bool) -> String {
        if flip_y_axis {
            self.elements
                .iter()
                .map(|element| match element {
                    PathElement::MoveTo(it) => PathElement::MoveTo(Vec2::new(it.x, -it.y)),
                    PathElement::Arc {
                        end,
                        large_arc,
                        radius,
                    } => PathElement::Arc {
                        end: Vec2::new(end.x, -end.y),
                        large_arc: *large_arc,
                        radius: *radius,
                    },
                })
                .map(|element| element.to_string())
                .join(" ")
        } else {
            self.elements
                .iter()
                .map(|element| element.to_string())
                .join(" ")
        }
    }
}

impl From<Vec<PathElement>> for Path {
    fn from(value: Vec<PathElement>) -> Self {
        Self {
            elements: value,
            class: Class::default(),
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let attributes = [
            format!("d=\"{}\"", self.path(true)),
            format!("{}", self.class),
        ];

        write!(
            f,
            "<path {}/>",
            attributes.into_iter().filter(|it| !it.is_empty()).join(" ")
        )
    }
}

impl Geometry for Path {
    fn add_geometry(&self, b: &mut Builder) {
        shapes::SvgPathShape {
            svg_doc_size_in_px: Default::default(),
            svg_path_string: self.path(false),
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

impl Indent for Path {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_path_tag_without_class() {
        let mut path = Path::new();
        path.push(PathElement::MoveTo(Vec2::X));
        path.push(PathElement::Arc {
            radius: 10.0,
            large_arc: true,
            end: Vec2::Y,
        });

        let result = format!("{}", path);

        let expected = r#"<path d="M 1 -0 A 10 10 0 1 1 0 -1"/>"#;

        assert_eq!(result, expected);
    }

    #[test]
    fn should_create_path_tag_with_class() {
        let mut path = Path::new();
        path.class = Class("foo".to_string());
        path.push(PathElement::MoveTo(Vec2::X));
        path.push(PathElement::Arc {
            radius: 10.0,
            large_arc: true,
            end: Vec2::Y,
        });

        let result = format!("{}", path);

        let expected = r#"<path d="M 1 -0 A 10 10 0 1 1 0 -1" class="foo"/>"#;

        assert_eq!(result, expected);
    }
}
