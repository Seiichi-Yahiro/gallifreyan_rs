#[derive(Debug, Clone)]
pub struct Style {
    rules: Vec<StyleRule>,
}

#[derive(Debug, Clone)]
pub struct StyleRule {
    selector: Selector,
    rules: Vec<CSSRule>,
}

impl Display for StyleRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let content = self.rules.iter().map();
        write!(f, "{}{{\n{}\n}}", self.selector, content)
    }
}

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
    Stroke(Color),
    Fill(Color),
    StrokeWidth(u32),
}

/*impl Display for DrawMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DrawMode::Fill(color) => {
                write!(
                    f,
                    "stroke=\"none\" fill=\"rgb({}, {}, {})\"",
                    color.r() * 255.0,
                    color.g() * 255.0,
                    color.b() * 255.0
                )
            }
            DrawMode::Stroke(color) => {
                write!(
                    f,
                    "stroke=\"rgb({}, {}, {})\" fill=\"none\"",
                    color.r() * 255.0,
                    color.g() * 255.0,
                    color.b() * 255.0
                )
            }
        }
    }
}*/
