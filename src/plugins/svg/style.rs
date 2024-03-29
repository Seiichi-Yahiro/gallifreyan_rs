use super::DEFAULT_INDENTATION_DEPTH;
use super::{Indent, ToCSSString};
use bevy::prelude::Color;
use itertools::Itertools;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone)]
pub struct Style(pub Vec<StyleRule>);

impl Display for Style {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let rules = self
            .0
            .iter()
            .map(|rule| rule.indent(DEFAULT_INDENTATION_DEPTH))
            .join("\n");

        write!(f, "<style>\n{}\n</style>", rules)
    }
}

impl Style {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, rule: StyleRule) {
        self.0.push(rule);
    }
}

impl Indent for Style {}

#[derive(Debug, Default, Clone)]
pub struct StyleRule {
    pub selectors: Vec<Selector>,
    pub rules: Vec<CSSRule>,
}

impl Display for StyleRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let selectors = self
            .selectors
            .iter()
            .map(|selector| selector.to_string())
            .join(",\n");

        let rules = self
            .rules
            .iter()
            .map(|rule| rule.indent(DEFAULT_INDENTATION_DEPTH))
            .join("\n");

        write!(f, "{} {{\n{}\n}}", selectors, rules)
    }
}

impl StyleRule {
    pub fn new() -> Self {
        Self {
            selectors: Vec::new(),
            rules: Vec::new(),
        }
    }
}

impl Indent for StyleRule {}

#[derive(Debug, Clone)]
pub enum Selector {
    Class(String),
    Tag(String),
}

impl Display for Selector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Selector::Class(class) => {
                write!(f, ".{}", class)
            }
            Selector::Tag(tag) => {
                write!(f, "{}", tag)
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum CSSRule {
    Stroke(Option<Color>),
    Fill(Option<Color>),
    StrokeWidth(f32),
    StrokeLineCap(StrokeLineCap),
}

impl Display for CSSRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CSSRule::Stroke(stroke) => {
                write!(f, "stroke: {};", stroke.to_css_string())
            }
            CSSRule::Fill(fill) => {
                write!(f, "fill: {};", fill.to_css_string())
            }
            CSSRule::StrokeWidth(width) => {
                write!(f, "stroke-width: {};", width)
            }
            CSSRule::StrokeLineCap(cap) => {
                write!(f, "stroke-linecap: {};", cap)
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum StrokeLineCap {
    Butt,
    Round,
    Square,
}

impl Display for StrokeLineCap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StrokeLineCap::Butt => {
                write!(f, "butt")
            }
            StrokeLineCap::Round => {
                write!(f, "round")
            }
            StrokeLineCap::Square => {
                write!(f, "square")
            }
        }
    }
}

impl Indent for CSSRule {}

impl ToCSSString for Option<Color> {
    fn to_css_string(&self) -> String {
        match self {
            Some(color) => {
                format!(
                    "rgb({}, {}, {})",
                    color.r() * 255.0,
                    color.g() * 255.0,
                    color.b() * 255.0
                )
            }
            None => "none".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Class(pub String);

impl Display for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(f, "")
        } else {
            write!(f, "class=\"{}\"", self.0)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_create_style_tag() {
        let mut style = Style::new();

        let mut style_rule_1 = StyleRule::new();
        style_rule_1
            .selectors
            .push(Selector::Class("word".to_string()));
        style_rule_1
            .selectors
            .push(Selector::Tag("circle".to_string()));
        style_rule_1.rules.push(CSSRule::Fill(None));
        style_rule_1.rules.push(CSSRule::Stroke(Some(Color::PINK)));
        style_rule_1.rules.push(CSSRule::StrokeWidth(2.0));
        style_rule_1
            .rules
            .push(CSSRule::StrokeLineCap(StrokeLineCap::Round));

        let mut style_rule_2 = StyleRule::new();
        style_rule_2
            .selectors
            .push(Selector::Class("foo".to_string()));

        style_rule_2.rules.push(CSSRule::Fill(Some(Color::PINK)));
        style_rule_2.rules.push(CSSRule::Stroke(None));

        style.push(style_rule_1);
        style.push(style_rule_2);

        let result = style.to_string();

        let expected = r#"<style>
    .word,
    circle {
        fill: none;
        stroke: rgb(255, 20.4, 147.9);
        stroke-width: 2;
        stroke-linecap: round;
    }
    .foo {
        fill: rgb(255, 20.4, 147.9);
        stroke: none;
    }
</style>"#;

        assert_eq!(result, expected);
    }
}
