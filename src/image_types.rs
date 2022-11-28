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
        placement: Placement,
    ) -> PositionData {
        let distance = match placement {
            Placement::OnLine => word_radius,
            Placement::Outside => word_radius + Self::radius(word_radius, number_of_letters) * 1.5,
            Placement::Inside => {
                if number_of_letters > 1 {
                    word_radius - Self::radius(word_radius, number_of_letters) * 1.5
                } else {
                    0.0
                }
            }
            _ => {
                panic!("{:?} is not a vocal placement!", placement);
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
        placement: Placement,
    ) -> PositionData {
        let distance = match placement {
            Placement::DeepCut => word_radius - Self::radius(word_radius, number_of_letters) * 0.75,
            Placement::Inside => {
                if number_of_letters > 1 {
                    word_radius - Self::radius(word_radius, number_of_letters) * 1.5
                } else {
                    0.0
                }
            }
            Placement::ShallowCut => word_radius,
            Placement::OnLine => word_radius,
            _ => {
                panic!("{:?} is not a consonant placement", placement);
            }
        };

        let angle = index as f32 * (360.0 / number_of_letters as f32);

        PositionData { distance, angle }
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct Dot;

impl Dot {
    pub fn radius(consonant_radius: f32) -> f32 {
        consonant_radius * 0.1
    }

    pub fn position_data(
        consonant_radius: f32,
        number_of_dots: usize,
        index: usize,
    ) -> PositionData {
        const LETTER_SIDE_ANGLE: f32 = 180.0;
        const DOT_DISTANCE_ANGLE: f32 = 45.0;

        let center_dots_on_letter_side_angle: f32 =
            ((number_of_dots - 1) as f32 * DOT_DISTANCE_ANGLE) / 2.0;

        let distance = consonant_radius - Self::radius(consonant_radius) * 1.5;

        let angle = index as f32 * DOT_DISTANCE_ANGLE - center_dots_on_letter_side_angle
            + LETTER_SIDE_ANGLE;

        PositionData { distance, angle }
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub struct LineSlot;

impl LineSlot {
    pub fn position_data(
        letter_radius: f32,
        number_of_lines: usize,
        index: usize,
        point_outside: bool,
    ) -> PositionData {
        let letter_side_angle = if point_outside { 0.0 } else { 180.0 };
        const LINE_DISTANCE_ANGLE: f32 = 45.0;
        let center_lines_on_letter_side_angle =
            ((number_of_lines - 1) as f32 * LINE_DISTANCE_ANGLE) / 2.0;

        let distance = letter_radius;

        let angle = index as f32 * LINE_DISTANCE_ANGLE - center_lines_on_letter_side_angle
            + letter_side_angle;

        PositionData { distance, angle }
    }
}

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
    pub placement: Placement,
    pub decoration: Decoration,
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
            placement: Placement::OnLine,
            decoration: Decoration::None,
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
        let placement = Placement::try_from(letter.as_str()).unwrap();

        Self {
            radius: Radius(Vocal::radius(word_radius, number_of_letters)),
            position_data: Vocal::position_data(word_radius, number_of_letters, index, placement),
            placement,
            decoration: Decoration::try_from(letter.as_str()).unwrap(),
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
    pub placement: Placement,
    pub decoration: Decoration,
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
            placement: Placement::DeepCut,
            decoration: Decoration::None,
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
        let placement = Placement::try_from(letter.as_str()).unwrap();

        Self {
            radius: Radius(Consonant::radius(word_radius, number_of_letters)),
            position_data: Consonant::position_data(
                word_radius,
                number_of_letters,
                index,
                placement,
            ),
            placement,
            decoration: Decoration::try_from(letter.as_str()).unwrap(),
            text: Text(letter),
            ..default()
        }
    }
}

#[derive(Bundle)]
pub struct DotBundle {
    pub dot: Dot,
    pub radius: Radius,
    pub position_data: PositionData,
    pub shape: ShapeBundle,
}

impl Default for DotBundle {
    fn default() -> Self {
        Self {
            dot: Dot,
            radius: Default::default(),
            position_data: Default::default(),
            shape: ShapeBundle {
                mode: DrawMode::Fill(FillMode::color(Color::BLACK)),
                ..default()
            },
        }
    }
}

impl DotBundle {
    pub fn new(consonant_radius: f32, number_of_dots: usize, index: usize) -> Self {
        Self {
            radius: Radius(Dot::radius(consonant_radius)),
            position_data: Dot::position_data(consonant_radius, number_of_dots, index),
            ..default()
        }
    }
}

#[derive(Bundle)]
pub struct LineSlotBundle {
    pub line_slot: LineSlot,
    pub position_data: PositionData,
    pub shape: ShapeBundle,
}

impl Default for LineSlotBundle {
    fn default() -> Self {
        Self {
            line_slot: LineSlot,
            position_data: PositionData::default(),
            shape: ShapeBundle {
                mode: DrawMode::Stroke(StrokeMode::new(Color::BLACK, LINE_WIDTH)),
                ..default()
            },
        }
    }
}

impl LineSlotBundle {
    pub fn new(
        letter_radius: f32,
        number_of_lines: usize,
        index: usize,
        point_outside: bool,
    ) -> Self {
        Self {
            position_data: LineSlot::position_data(
                letter_radius,
                number_of_lines,
                index,
                point_outside,
            ),
            ..default()
        }
    }
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
pub enum Placement {
    DeepCut,    // c
    Inside,     // cv
    ShallowCut, // c
    OnLine,     // cv
    Outside,    // v
}

impl TryFrom<&str> for Placement {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let placement = match value {
            "b" | "ch" | "d" | "g" | "h" | "f" => Placement::DeepCut,
            "j" | "ph" | "k" | "l" | "c" | "n" | "p" | "m" => Placement::Inside,
            "t" | "wh" | "sh" | "r" | "v" | "w" | "s" => Placement::ShallowCut,
            "th" | "gh" | "y" | "z" | "q" | "qu" | "x" | "ng" => Placement::OnLine,
            "o" => Placement::Inside,
            "a" => Placement::Outside,
            "e" | "i" | "u" => Placement::OnLine,
            _ => {
                return Err(format!(
                    "Cannot assign placement to '{}' as it is not a valid letter!",
                    value
                ))
            }
        };

        Ok(placement)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component)]
pub enum Decoration {
    None,         // cv
    SingleDot,    // c
    DoubleDot,    // c
    TripleDot,    // c
    QuadrupleDot, // c
    SingleLine,   // c
    DoubleLine,   // c
    TripleLine,   // c
    LineInside,   // v
    LineOutside,  // v
}

impl TryFrom<&str> for Decoration {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let decoration = match value {
            "b" | "j" | "t" | "th" => Decoration::None,
            "ph" | "wh" | "gh" => Decoration::SingleDot,
            "ch" | "k" | "sh" | "y" => Decoration::DoubleDot,
            "d" | "l" | "r" | "z" => Decoration::TripleDot,
            "c" | "q" => Decoration::QuadrupleDot,
            "g" | "n" | "v" | "qu" => Decoration::SingleLine,
            "h" | "p" | "w" | "x" => Decoration::DoubleLine,
            "f" | "m" | "s" | "ng" => Decoration::TripleLine,
            "i" => Decoration::LineInside,
            "u" => Decoration::LineOutside,
            "a" | "e" | "o" => Decoration::None,
            _ => {
                return Err(format!(
                    "Cannot assign decoration to '{}' as it is not a valid letter!",
                    value
                ))
            }
        };

        Ok(decoration)
    }
}

impl Decoration {
    pub fn dots(&self) -> usize {
        match self {
            Decoration::SingleDot => 1,
            Decoration::DoubleDot => 2,
            Decoration::TripleDot => 3,
            Decoration::QuadrupleDot => 4,
            _ => 0,
        }
    }

    pub fn lines(&self) -> usize {
        match self {
            Decoration::SingleLine | Decoration::LineInside | Decoration::LineOutside => 1,
            Decoration::DoubleLine => 2,
            Decoration::TripleLine => 3,
            _ => 0,
        }
    }

    pub fn line_points_outside(&self) -> bool {
        match self {
            Decoration::LineInside => false,
            _ => true,
        }
    }
}
