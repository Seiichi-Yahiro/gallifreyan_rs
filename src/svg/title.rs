use super::Indent;
use bevy::prelude::{FromReflect, Reflect};
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone, Reflect, FromReflect)]
pub struct Title {
    pub text: String,
}

impl Display for Title {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<title>{}</title>", self.text)
    }
}

impl Indent for Title {}
