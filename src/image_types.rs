use bevy::prelude::*;

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

#[derive(Bundle)]
pub struct SentenceBundle {
    pub sentence: Sentence,
    pub text: Text,
    pub radius: Radius,
    pub position_data: PositionData,
    pub words: CircleChildren,
    pub line_slots: LineSlotChildren,
}

#[derive(Bundle)]
pub struct WordBundle {
    pub word: Word,
    pub text: Text,
    pub radius: Radius,
    pub position_data: PositionData,
    pub letters: CircleChildren,
    pub line_slots: LineSlotChildren,
}

#[derive(Bundle)]
pub struct VocalBundle {
    pub vocal: Vocal,
    pub letter: Letter,
    pub text: Text,
    pub radius: Radius,
    pub position_data: PositionData,
    pub placement: VocalPlacement,
    pub decoration: VocalDecoration,
    pub line_slots: LineSlotChildren,
}

#[derive(Bundle)]
pub struct ConsonantBundle {
    pub consonant: Consonant,
    pub letter: Letter,
    pub text: Text,
    pub radius: Radius,
    pub position_data: PositionData,
    pub placement: ConsonantPlacement,
    pub decoration: ConsonantDecoration,
    pub dots: CircleChildren,
    pub line_slots: LineSlotChildren,
}

#[derive(Bundle)]
pub struct DotBundle {
    pub dot: Dot,
    pub radius: Radius,
    pub position_data: PositionData,
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
