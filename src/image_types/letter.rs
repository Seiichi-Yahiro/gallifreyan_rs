use crate::image_types::{
    new_stroke_mode, AnglePlacement, CircleChildren, LineSlotChildren, PositionData, Radius, Text,
};
use crate::math::Angle;
use crate::style::Styles;
use crate::svg_view::Interaction;
use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::DrawMode;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Component, Reflect)]
#[reflect(Component)]
pub enum Letter {
    Vocal,
    #[default]
    Consonant,
}

impl Letter {
    pub fn radius(&self, word_radius: f32, number_of_letters: usize) -> f32 {
        match self {
            Letter::Vocal => (word_radius * 0.75 * 0.4) / (1.0 + number_of_letters as f32 / 2.0),
            Letter::Consonant => (word_radius * 0.75) / (1.0 + number_of_letters as f32 / 2.0),
        }
    }

    pub fn position_data(
        &self,
        word_radius: f32,
        number_of_letters: usize,
        index: usize,
        placement: Placement,
    ) -> PositionData {
        match self {
            Letter::Vocal => {
                let distance = match placement {
                    Placement::OnLine => word_radius,
                    Placement::Outside => {
                        word_radius + self.radius(word_radius, number_of_letters) * 1.5
                    }
                    Placement::Inside => {
                        if number_of_letters > 1 {
                            word_radius - self.radius(word_radius, number_of_letters) * 1.5
                        } else {
                            0.0
                        }
                    }
                    _ => {
                        panic!("{:?} is not a vocal placement!", placement);
                    }
                };

                let angle = index as f32 * (360.0 / number_of_letters as f32);

                PositionData {
                    distance,
                    angle: Angle::new_degree(angle),
                    angle_placement: AnglePlacement::Relative,
                }
            }
            Letter::Consonant => {
                let distance = match placement {
                    Placement::DeepCut => {
                        word_radius - self.radius(word_radius, number_of_letters) * 0.75
                    }
                    Placement::Inside => {
                        if number_of_letters > 1 {
                            word_radius - self.radius(word_radius, number_of_letters) * 1.5
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

                PositionData {
                    distance,
                    angle: Angle::new_degree(angle),
                    angle_placement: AnglePlacement::Relative,
                }
            }
        }
    }
}

#[derive(Bundle)]
pub struct LetterBundle {
    pub letter: Letter,
    pub text: Text,
    pub radius: Radius,
    pub position_data: PositionData,
    pub placement: Placement,
    pub decoration: Decoration,
    pub dots: CircleChildren,
    pub line_slots: LineSlotChildren,
    pub interaction: Interaction,
}

impl LetterBundle {
    pub fn new(
        letter: Letter,
        letter_text: String,
        word_radius: f32,
        number_of_letters: usize,
        index: usize,
    ) -> Self {
        let placement = Placement::try_from(letter_text.as_str()).unwrap();

        Self {
            letter,
            radius: Radius(letter.radius(word_radius, number_of_letters)),
            position_data: letter.position_data(word_radius, number_of_letters, index, placement),
            placement,
            decoration: Decoration::try_from(letter_text.as_str()).unwrap(),
            dots: Default::default(),
            text: Text(letter_text),
            line_slots: Default::default(),
            interaction: Interaction::default(),
        }
    }
}

// needed for reflection
pub fn add_shape_for_letter(
    mut commands: Commands,
    query: Query<Entity, Added<Letter>>,
    styles: Res<Styles>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(ShapeBundle {
            mode: DrawMode::Stroke(new_stroke_mode(styles.svg_color)),
            transform: Transform::from_xyz(0.0, 0.0, 0.1),
            ..default()
        });
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Component, Reflect)]
#[reflect(Component)]
pub enum Placement {
    DeepCut,    // c
    Inside,     // cv
    ShallowCut, // c
    #[default]
    OnLine, // cv
    Outside,    // v
}

impl TryFrom<&str> for Placement {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let placement = match value.to_ascii_lowercase().as_str() {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Component, Reflect)]
#[reflect(Component)]
pub enum Decoration {
    #[default]
    None, // cv
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
        let decoration = match value.to_ascii_lowercase().as_str() {
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
        *self == Decoration::LineOutside
    }
}
