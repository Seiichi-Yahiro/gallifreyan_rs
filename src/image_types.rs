use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::{DrawMode, FillMode, StrokeMode};

#[derive(Debug, Copy, Clone, Component)]
pub struct Sentence;

impl Sentence {
    pub fn radius() -> f32 {
        (1000.0 / 2.0) * 0.9
    }

    pub fn position_data() -> PositionData {
        PositionData {
            angle: 0.0,
            distance: 0.0,
        }
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct Word;

impl Word {
    pub fn radius(sentence_radius: f32, number_of_words: usize) -> f32 {
        (sentence_radius * 0.75) / (1.0 + number_of_words as f32 / 2.0)
    }

    pub fn position_data(
        sentence_radius: f32,
        number_of_words: usize,
        index: usize,
    ) -> PositionData {
        PositionData {
            distance: if number_of_words > 1 {
                sentence_radius - Self::radius(sentence_radius, number_of_words) * 1.5
            } else {
                0.0
            },
            angle: index as f32 * (360.0 / number_of_words as f32),
        }
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct Letter;

#[derive(Debug, Copy, Clone, Component)]
pub struct Vocal;

impl Vocal {
    pub fn radius(word_radius: f32, number_of_letters: usize) -> f32 {
        (word_radius * 0.75 * 0.4) / (1.0 + number_of_letters as f32 / 2.0)
    }

    pub fn position_data(
        word_radius: f32,
        number_of_letters: usize,
        index: usize,
        placement: VocalPlacement,
    ) -> PositionData {
        let distance = match placement {
            VocalPlacement::OnLine => word_radius,
            VocalPlacement::Outside => {
                word_radius + Self::radius(word_radius, number_of_letters) * 1.5
            }
            VocalPlacement::Inside => {
                if number_of_letters > 1 {
                    word_radius - Self::radius(word_radius, number_of_letters) * 1.5
                } else {
                    0.0
                }
            }
        };

        let angle = index as f32 * (360.0 / number_of_letters as f32);

        PositionData { distance, angle }
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct Consonant;

impl Consonant {
    pub fn radius(word_radius: f32, number_of_letters: usize) -> f32 {
        (word_radius * 0.75) / (1.0 + number_of_letters as f32 / 2.0)
    }

    pub fn position_data(
        word_radius: f32,
        number_of_letters: usize,
        index: usize,
        placement: ConsonantPlacement,
    ) -> PositionData {
        let distance = match placement {
            ConsonantPlacement::DeepCut => {
                word_radius - Self::radius(word_radius, number_of_letters) * 0.75
            }
            ConsonantPlacement::Inside => {
                if number_of_letters > 1 {
                    word_radius - Self::radius(word_radius, number_of_letters) * 1.5
                } else {
                    0.0
                }
            }
            ConsonantPlacement::ShallowCut => word_radius,
            ConsonantPlacement::OnLine => word_radius,
        };

        let angle = index as f32 * (360.0 / number_of_letters as f32);

        PositionData { distance, angle }
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct Dot;
#[derive(Debug, Copy, Clone, Component)]
pub struct LineSlot;

#[derive(Default, Component, Deref, DerefMut)]
pub struct CircleChildren(pub Vec<Entity>);

#[derive(Default, Component, Deref, DerefMut)]
pub struct LineSlotChildren(pub Vec<Entity>);

const LINE_WIDTH: f32 = 1.0;

#[derive(Bundle)]
pub struct SentenceBundle {
    pub sentence: Sentence,
    pub text: Text,
    pub radius: Radius,
    pub position_data: PositionData,
    pub words: CircleChildren,
    pub line_slots: LineSlotChildren,
    pub shape: ShapeBundle,
}

impl Default for SentenceBundle {
    fn default() -> Self {
        Self {
            sentence: Sentence,
            text: Text::default(),
            radius: Default::default(),
            position_data: Default::default(),
            words: CircleChildren::default(),
            line_slots: LineSlotChildren::default(),
            shape: ShapeBundle {
                mode: DrawMode::Stroke(StrokeMode::new(Color::BLACK, LINE_WIDTH)),
                ..default()
            },
        }
    }
}

impl SentenceBundle {
    pub fn new(sentence: String) -> Self {
        Self {
            text: Text(sentence),
            radius: Radius(Sentence::radius()),
            position_data: Sentence::position_data(),
            ..default()
        }
    }
}

#[derive(Bundle)]
pub struct WordBundle {
    pub word: Word,
    pub text: Text,
    pub radius: Radius,
    pub position_data: PositionData,
    pub letters: CircleChildren,
    pub line_slots: LineSlotChildren,
    pub shape: ShapeBundle,
}

impl Default for WordBundle {
    fn default() -> Self {
        Self {
            word: Word,
            text: Text::default(),
            radius: Default::default(),
            position_data: Default::default(),
            letters: CircleChildren::default(),
            line_slots: LineSlotChildren::default(),
            shape: ShapeBundle {
                mode: DrawMode::Stroke(StrokeMode::new(Color::BLACK, LINE_WIDTH)),
                ..default()
            },
        }
    }
}

impl WordBundle {
    pub fn new(word: String, sentence_radius: f32, number_of_words: usize, index: usize) -> Self {
        Self {
            text: Text(word),
            radius: Radius(Word::radius(sentence_radius, number_of_words)),
            position_data: Word::position_data(sentence_radius, number_of_words, index),
            ..default()
        }
    }
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
    pub shape: ShapeBundle,
}

impl Default for VocalBundle {
    fn default() -> Self {
        Self {
            vocal: Vocal,
            letter: Letter,
            text: Default::default(),
            radius: Default::default(),
            position_data: Default::default(),
            placement: VocalPlacement::OnLine,
            decoration: VocalDecoration::None,
            line_slots: Default::default(),
            shape: ShapeBundle {
                mode: DrawMode::Stroke(StrokeMode::new(Color::BLACK, LINE_WIDTH)),
                ..default()
            },
        }
    }
}

impl VocalBundle {
    pub fn new(letter: String, word_radius: f32, number_of_letters: usize, index: usize) -> Self {
        let placement = VocalPlacement::try_from(letter.as_str()).unwrap();

        Self {
            radius: Radius(Vocal::radius(word_radius, number_of_letters)),
            position_data: Vocal::position_data(word_radius, number_of_letters, index, placement),
            placement,
            decoration: VocalDecoration::try_from(letter.as_str()).unwrap(),
            text: Text(letter),
            ..default()
        }
    }
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
    pub shape: ShapeBundle,
}

impl Default for ConsonantBundle {
    fn default() -> Self {
        Self {
            consonant: Consonant,
            letter: Letter,
            text: Default::default(),
            radius: Default::default(),
            position_data: Default::default(),
            placement: ConsonantPlacement::DeepCut,
            decoration: ConsonantDecoration::None,
            dots: Default::default(),
            line_slots: Default::default(),
            shape: ShapeBundle {
                mode: DrawMode::Stroke(StrokeMode::new(Color::BLACK, LINE_WIDTH)),
                ..default()
            },
        }
    }
}

impl ConsonantBundle {
    pub fn new(letter: String, word_radius: f32, number_of_letters: usize, index: usize) -> Self {
        let placement = ConsonantPlacement::try_from(letter.as_str()).unwrap();

        Self {
            radius: Radius(Consonant::radius(word_radius, number_of_letters)),
            position_data: Consonant::position_data(
                word_radius,
                number_of_letters,
                index,
                placement,
            ),
            placement,
            decoration: ConsonantDecoration::try_from(letter.as_str()).unwrap(),
            text: Text(letter),
            ..default()
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
            radius: Radius::default(),
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

#[derive(Debug, Default, PartialEq, Eq, Component, Deref, DerefMut)]
pub struct Text(pub String);

#[derive(Debug, Default, Copy, Clone, PartialEq, Component, Deref, DerefMut)]
pub struct Radius(pub f32);

#[derive(Debug, Default, Copy, Clone, PartialEq, Component)]
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
