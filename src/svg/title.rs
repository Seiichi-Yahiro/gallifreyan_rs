use super::Indent;
use bevy::prelude::{FromReflect, Reflect};
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone, Reflect, FromReflect)]
pub struct Title(pub String);


impl Display for Title {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<title>{}</title>", self.0)
    }
}

impl Indent for Title {}
