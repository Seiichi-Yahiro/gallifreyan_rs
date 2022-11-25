use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::{DrawMode, FillMode, StrokeMode};

#[derive(Debug, Copy, Clone, Component)]
pub struct Sentence;
#[derive(Debug, Copy, Clone, Component)]
pub struct Word;
#[derive(Debug, Copy, Clone, Component)]
pub struct Letter;
#[derive(Debug, Copy, Clone, Component)]
pub struct Vocal;
#[derive(Debug, Copy, Clone, Component)]
pub struct Consonant;
#[derive(Debug, Copy, Clone, Component)]
pub struct Dot;
#[derive(Debug, Copy, Clone, Component)]
pub struct LineSlot;

#[derive(Component, Deref, DerefMut)]
pub struct CircleChildren(pub Vec<Entity>);

#[derive(Component, Deref, DerefMut)]
pub struct LineSlotChildren(pub Vec<Entity>);

const LINE_WIDTH: f32 = 4.0;

#[derive(Bundle)]
pub struct SentenceBundle {
    sentence: Sentence,
    text: Text,
    radius: Radius,
    position_data: PositionData,
    words: CircleChildren,
    line_slots: LineSlotChildren,
    shape: ShapeBundle,
}

impl SentenceBundle {
    pub fn new(text: Text, words: CircleChildren, line_slots: LineSlotChildren) -> Self {
        Self {
            sentence: Sentence,
            text,
            radius: Radius(500.0),
            position_data: PositionData::default(),
            words,
            line_slots,
            shape: ShapeBundle {
                mode: DrawMode::Stroke(StrokeMode::new(Color::BLACK, LINE_WIDTH)),
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct WordBundle {
    word: Word,
    text: Text,
    radius: Radius,
    position_data: PositionData,
    letters: CircleChildren,
    line_slots: LineSlotChildren,
    shape: ShapeBundle,
}

impl WordBundle {
    pub fn new(text: Text, letters: CircleChildren, line_slots: LineSlotChildren) -> Self {
        Self {
            word: Word,
            text,
            radius: Radius(100.0),
            position_data: PositionData::default(),
            letters,
            line_slots,
            shape: ShapeBundle {
                mode: DrawMode::Stroke(StrokeMode::new(Color::BLACK, LINE_WIDTH)),
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct VocalBundle {
    vocal: Vocal,
    letter: Letter,
    text: Text,
    radius: Radius,
    position_data: PositionData,
    placement: VocalPlacement,
    decoration: VocalDecoration,
    line_slots: LineSlotChildren,
    shape: ShapeBundle,
}

impl VocalBundle {
    pub fn new(
        text: Text,
        placement: VocalPlacement,
        decoration: VocalDecoration,
        line_slots: LineSlotChildren,
    ) -> Self {
        Self {
            vocal: Vocal,
            letter: Letter,
            text,
            radius: Radius(25.0),
            position_data: PositionData::default(),
            placement,
            decoration,
            line_slots,
            shape: ShapeBundle {
                mode: DrawMode::Stroke(StrokeMode::new(Color::BLACK, LINE_WIDTH)),
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct ConsonantBundle {
    consonant: Consonant,
    letter: Letter,
    text: Text,
    radius: Radius,
    position_data: PositionData,
    placement: ConsonantPlacement,
    decoration: ConsonantDecoration,
    dots: CircleChildren,
    line_slots: LineSlotChildren,
    shape: ShapeBundle,
}

impl ConsonantBundle {
    pub fn new(
        text: Text,
        placement: ConsonantPlacement,
        decoration: ConsonantDecoration,
        dots: CircleChildren,
        line_slots: LineSlotChildren,
    ) -> Self {
        Self {
            consonant: Consonant,
            letter: Letter,
            text,
            radius: Radius(50.0),
            position_data: PositionData::default(),
            placement,
            decoration,
            dots,
            line_slots,
            shape: ShapeBundle {
                mode: DrawMode::Stroke(StrokeMode::new(Color::BLACK, LINE_WIDTH)),
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct DotBundle {
    dot: Dot,
    radius: Radius,
    position_data: PositionData,
    shape: ShapeBundle,
}

impl DotBundle {
    pub fn new() -> Self {
        Self {
            dot: Dot,
            radius: Radius(5.0),
            position_data: PositionData::default(),
            shape: ShapeBundle {
                mode: DrawMode::Fill(FillMode::color(Color::BLACK)),
                ..default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct LineSlotBundle {
    pub line_slot: LineSlot,
    pub position_data: PositionData,
}

#[derive(Debug, Component, Deref, DerefMut)]
pub struct Text(pub String);

#[derive(Debug, Default, Copy, Clone, Component, Deref, DerefMut)]
pub struct Radius(pub f32);

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct PositionData {
    pub angle: f32,
    pub distance: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component)]
pub enum ConsonantPlacement {
    DeepCut,
    Inside,
    ShallowCut,
    OnLine,
}

impl TryFrom<&str> for ConsonantPlacement {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let placement = match value {
            "b" | "ch" | "d" | "g" | "h" | "f" => ConsonantPlacement::DeepCut,
            "j" | "ph" | "k" | "l" | "c" | "n" | "p" | "m" => ConsonantPlacement::Inside,
            "t" | "wh" | "sh" | "r" | "v" | "w" | "s" => ConsonantPlacement::ShallowCut,
            "th" | "gh" | "y" | "z" | "q" | "qu" | "x" | "ng" => ConsonantPlacement::OnLine,
            _ => {
                return Err(format!(
                    "Cannot assign consonant placement to '{}' as it is not a consonant!",
                    value
                ))
            }
        };

        Ok(placement)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component)]
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

impl TryFrom<&str> for ConsonantDecoration {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let decoration = match value {
            "b" | "j" | "t" | "th" => ConsonantDecoration::None,
            "ph" | "wh" | "gh" => ConsonantDecoration::SingleDot,
            "ch" | "k" | "sh" | "y" => ConsonantDecoration::DoubleDot,
            "d" | "l" | "r" | "z" => ConsonantDecoration::TripleDot,
            "c" | "q" => ConsonantDecoration::QuadrupleDot,
            "g" | "n" | "v" | "qu" => ConsonantDecoration::SingleLine,
            "h" | "p" | "w" | "x" => ConsonantDecoration::DoubleLine,
            "f" | "m" | "s" | "ng" => ConsonantDecoration::TripleLine,
            _ => {
                return Err(format!(
                    "Cannot assign consonant decoration to '{}' as it is not a consonant!",
                    value
                ))
            }
        };

        Ok(decoration)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component)]
pub enum VocalPlacement {
    OnLine,
    Outside,
    Inside,
}

impl TryFrom<&str> for VocalPlacement {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let placement = match value {
            "o" => VocalPlacement::Inside,
            "a" => VocalPlacement::Outside,
            "e" | "i" | "u" => VocalPlacement::OnLine,
            _ => {
                return Err(format!(
                    "Cannot assign vocal placement to '{}' as it is not a vocal!",
                    value
                ))
            }
        };

        Ok(placement)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component)]
pub enum VocalDecoration {
    None,
    LineInside,
    LineOutside,
}

impl TryFrom<&str> for VocalDecoration {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let decoration = match value {
            "i" => VocalDecoration::LineInside,
            "u" => VocalDecoration::LineOutside,
            "a" | "e" | "o" => VocalDecoration::None,
            _ => {
                return Err(format!(
                    "Cannot assign vocal decoration to '{}' as it is not a vocal!",
                    value
                ))
            }
        };

        Ok(decoration)
    }
}
