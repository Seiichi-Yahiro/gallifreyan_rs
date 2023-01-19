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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_title_tag() {
        let result = format!("{}", Title("TITLE".to_string()));
        let expected = "<title>TITLE</title>";

        assert_eq!(result, expected);
    }
}
