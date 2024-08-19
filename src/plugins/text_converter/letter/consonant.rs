use super::Decorated;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ConsonantCluster {
    Single(Consonant),
    Digraph(Digraph),
}

impl TryFrom<&str> for ConsonantCluster {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.len() {
            1 => Consonant::try_from(value).map(ConsonantCluster::Single),
            2 => Digraph::try_from(value).map(ConsonantCluster::Digraph),
            _ => Err(format!("'{}' is not a valid consonant!", value)),
        }
    }
}

impl From<Consonant> for ConsonantCluster {
    fn from(value: Consonant) -> Self {
        Self::Single(value)
    }
}

impl From<Digraph> for ConsonantCluster {
    fn from(value: Digraph) -> Self {
        Self::Digraph(value)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Consonant {
    B,
    J,
    T,
    K,
    Y,
    D,
    L,
    R,
    Z,
    C,
    Q,
    G,
    N,
    V,
    H,
    P,
    W,
    X,
    F,
    M,
    S,
}

impl TryFrom<&str> for Consonant {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let consonant = match value.to_ascii_lowercase().as_str() {
            "b" => Self::B,
            "j" => Self::J,
            "t" => Self::T,
            "k" => Self::K,
            "y" => Self::Y,
            "d" => Self::D,
            "l" => Self::L,
            "r" => Self::R,
            "z" => Self::Z,
            "c" => Self::C,
            "q" => Self::Q,
            "g" => Self::G,
            "n" => Self::N,
            "v" => Self::V,
            "h" => Self::H,
            "p" => Self::P,
            "w" => Self::W,
            "x" => Self::X,
            "f" => Self::F,
            "m" => Self::M,
            "s" => Self::S,
            _ => return Err(format!("'{}' is not a valid grapheme!", value)),
        };

        Ok(consonant)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Digraph {
    TH,
    PH,
    WH,
    GH,
    CH,
    SH,
    QU,
    NG,
}

impl TryFrom<&str> for Digraph {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let digraph = match value.to_ascii_lowercase().as_str() {
            "th" => Self::TH,
            "ph" => Self::PH,
            "wh" => Self::WH,
            "gh" => Self::GH,
            "ch" => Self::CH,
            "sh" => Self::SH,
            "qu" => Self::QU,
            "ng" => Self::NG,
            _ => return Err(format!("'{}' is not a valid digraph!", value)),
        };

        Ok(digraph)
    }
}

pub trait ConsonantPlacementT: Into<ConsonantPlacement> {
    fn placement(&self) -> ConsonantPlacement;
}

impl<T> ConsonantPlacementT for T
where
    T: Into<ConsonantPlacement> + Copy,
{
    fn placement(&self) -> ConsonantPlacement {
        (*self).into()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ConsonantPlacement {
    DeepCut,
    Inside,
    ShallowCut,
    OnLine,
}

impl From<Consonant> for ConsonantPlacement {
    fn from(value: Consonant) -> Self {
        match value {
            Consonant::B | Consonant::D | Consonant::G | Consonant::H | Consonant::F => {
                Self::DeepCut
            }
            Consonant::J
            | Consonant::K
            | Consonant::L
            | Consonant::C
            | Consonant::N
            | Consonant::P
            | Consonant::M => Self::Inside,
            Consonant::T | Consonant::R | Consonant::V | Consonant::W | Consonant::S => {
                Self::ShallowCut
            }
            Consonant::Y | Consonant::Z | Consonant::Q | Consonant::X => Self::OnLine,
        }
    }
}

impl From<Digraph> for ConsonantPlacement {
    fn from(value: Digraph) -> Self {
        match value {
            Digraph::PH => Self::Inside,
            Digraph::CH => Self::DeepCut,
            Digraph::WH | Digraph::SH => Self::ShallowCut,
            Digraph::TH | Digraph::GH | Digraph::QU | Digraph::NG => Self::OnLine,
        }
    }
}

impl From<ConsonantCluster> for ConsonantPlacement {
    fn from(value: ConsonantCluster) -> Self {
        match value {
            ConsonantCluster::Single(grapheme) => grapheme.into(),
            ConsonantCluster::Digraph(digraph) => digraph.into(),
        }
    }
}

pub trait ConsonantDecorationT: Into<ConsonantDecoration> {
    fn decoration(&self) -> ConsonantDecoration;
}

impl<T> ConsonantDecorationT for T
where
    T: Into<ConsonantDecoration> + Copy,
{
    fn decoration(&self) -> ConsonantDecoration {
        (*self).into()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ConsonantDecoration {
    None,
    SingleDot,
    DoubleDot,
    TripleDot,
    QuadrupleDot,
    SingleLine,
    DoubleLine,
    TripleLine,
}

impl From<Consonant> for ConsonantDecoration {
    fn from(value: Consonant) -> Self {
        match value {
            Consonant::B | Consonant::J | Consonant::T => Self::None,
            Consonant::K | Consonant::Y => Self::DoubleDot,
            Consonant::D | Consonant::L | Consonant::R | Consonant::Z => Self::TripleDot,
            Consonant::C | Consonant::Q => Self::QuadrupleDot,
            Consonant::G | Consonant::N | Consonant::V => Self::SingleLine,
            Consonant::H | Consonant::P | Consonant::W | Consonant::X => Self::DoubleLine,
            Consonant::F | Consonant::M | Consonant::S => Self::TripleLine,
        }
    }
}

impl From<Digraph> for ConsonantDecoration {
    fn from(value: Digraph) -> Self {
        match value {
            Digraph::TH => Self::None,
            Digraph::PH | Digraph::WH | Digraph::GH => Self::SingleDot,
            Digraph::CH | Digraph::SH => Self::DoubleDot,
            Digraph::QU => Self::SingleLine,
            Digraph::NG => Self::TripleLine,
        }
    }
}

impl From<ConsonantCluster> for ConsonantDecoration {
    fn from(value: ConsonantCluster) -> Self {
        match value {
            ConsonantCluster::Single(grapheme) => grapheme.into(),
            ConsonantCluster::Digraph(digraph) => digraph.into(),
        }
    }
}

impl Decorated for ConsonantDecoration {
    fn dots(&self) -> usize {
        match self {
            Self::SingleDot => 1,
            Self::DoubleDot => 2,
            Self::TripleDot => 3,
            Self::QuadrupleDot => 4,
            _ => 0,
        }
    }

    fn lines(&self) -> usize {
        match self {
            Self::SingleLine => 1,
            Self::DoubleLine => 2,
            Self::TripleLine => 3,
            _ => 0,
        }
    }
}
