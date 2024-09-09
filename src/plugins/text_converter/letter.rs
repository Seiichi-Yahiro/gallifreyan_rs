use crate::math::angle::Degree;
use crate::plugins::text_converter::components::AnglePlacement;
use crate::plugins::text_converter::prelude::*;
use bevy::prelude::*;
use std::cmp::Ordering;
use std::fmt;

pub mod combinator;
pub mod consonant;
pub mod vocal;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Component)]
pub enum Letter {
    Vocal(Vocal),
    Consonant(ConsonantCluster),
}

impl fmt::Display for Letter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Letter::Vocal(v) => write!(f, "{}", v),
            Letter::Consonant(cluster) => write!(f, "{}", cluster),
        }
    }
}

impl PartialOrd for Letter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Letter {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

impl TryFrom<&str> for Letter {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Vocal::try_from(value)
            .map(Self::Vocal)
            .or_else(|_| ConsonantCluster::try_from(value).map(Self::Consonant))
            .map_err(|_| {
                format!(
                    "Cannot assign letter to '{}' as it is not a valid letter!",
                    value
                )
            })
    }
}

impl From<ConsonantCluster> for Letter {
    fn from(value: ConsonantCluster) -> Self {
        Self::Consonant(value)
    }
}

impl From<Consonant> for Letter {
    fn from(value: Consonant) -> Self {
        ConsonantCluster::Single(value).into()
    }
}

impl From<Digraph> for Letter {
    fn from(value: Digraph) -> Self {
        ConsonantCluster::Digraph(value).into()
    }
}

impl From<Vocal> for Letter {
    fn from(value: Vocal) -> Self {
        Self::Vocal(value)
    }
}

#[derive(Bundle)]
pub struct LetterBundle {
    pub name: Name,
    pub letter: Letter,
    pub text: GFText,
    pub dots: CircleChildren,
    pub line_slots: LineSlotChildren,
    pub sibling_index: SiblingIndex,
    pub radius: Radius,
    pub position_data: PositionData,
}

impl LetterBundle {
    pub fn new(text: String, letter: Letter, sibling_index: usize) -> Self {
        Self {
            name: Name::new("Letter"),
            letter,
            text: GFText(text),
            dots: Default::default(),
            line_slots: Default::default(),
            sibling_index: SiblingIndex(sibling_index),
            radius: Radius::default(),
            position_data: PositionData {
                angle_placement: AnglePlacement::Relative,
                ..default()
            },
        }
    }
}

impl Letter {
    pub fn is_cutting(&self) -> bool {
        match self {
            Self::Consonant(consonant) => match ConsonantPlacement::from(*consonant) {
                ConsonantPlacement::DeepCut | ConsonantPlacement::ShallowCut => true,
                ConsonantPlacement::OnLine | ConsonantPlacement::Inside => false,
            },
            Self::Vocal(_) => false,
        }
    }
}

pub trait Decorated {
    fn dots(&self) -> usize;
    fn lines(&self) -> usize;
}

impl Decorated for Letter {
    fn dots(&self) -> usize {
        match self {
            Letter::Vocal(vocal) => VocalDecoration::from(*vocal).dots(),
            Letter::Consonant(consonant) => ConsonantDecoration::from(*consonant).dots(),
        }
    }

    fn lines(&self) -> usize {
        match self {
            Letter::Vocal(vocal) => VocalDecoration::from(*vocal).lines(),
            Letter::Consonant(consonant) => ConsonantDecoration::from(*consonant).lines(),
        }
    }
}

fn split_word_to_letters(word: &str) -> impl Iterator<Item = LetterText> + '_ {
    // assume word is sanitized
    word.chars()
        .map(|grapheme| {
            let text = grapheme.to_string();

            LetterText {
                letter: Letter::try_from(text.as_str()).unwrap(),
                text,
            }
        })
        .combine_letters()
}

pub fn convert_letters(
    mut commands: Commands,
    mut word_query: Query<(Entity, &GFText, &mut CircleChildren), (With<Word>, Changed<GFText>)>,
    mut letter_query: Query<(Entity, &mut GFText, &mut Letter, &mut SiblingIndex), Without<Word>>,
) {
    for (word_entity, word_text, mut children) in word_query.iter_mut() {
        let mut existing_letters = letter_query.iter_many_mut(children.0.iter());
        let mut new_letters_iter = split_word_to_letters(&word_text.0).combine_letters();

        let mut new_children: Vec<Entity> = Vec::new();

        loop {
            let next_existing_letter = existing_letters.fetch_next();
            let next_new_letter = new_letters_iter.next();
            let sibling_index = new_children.len();

            match (next_existing_letter, next_new_letter) {
                // update letter
                (
                    Some((letter_entity, mut text, mut letter, mut letter_sibling_index)),
                    Some(LetterText {
                        text: new_text,
                        letter: new_letter,
                    }),
                ) => {
                    if text.0 != new_text {
                        debug!(
                            "Update letter: {:?} -> {:?}, {:?}",
                            *letter, new_letter, letter_entity
                        );

                        text.0 = new_text;
                        *letter = new_letter;
                    }

                    letter_sibling_index.0 = sibling_index;
                    new_children.push(letter_entity);
                }
                // remove letter
                (Some((letter_entity, _text, letter, _letter_sibling_index)), None) => {
                    debug!("Despawn letter: {:?}, {:?}", *letter, letter_entity);

                    commands.entity(letter_entity).despawn_recursive();
                }
                // add letter
                (
                    None,
                    Some(LetterText {
                        text: new_text,
                        letter: new_letter,
                    }),
                ) => {
                    let bundle = LetterBundle::new(new_text, new_letter, sibling_index);

                    let letter_entity = commands.spawn(bundle).id();
                    debug!("Spawn letter: {:?}, {:?}", new_letter, letter_entity);

                    commands.entity(word_entity).add_child(letter_entity);
                    new_children.push(letter_entity);
                }
                (None, None) => {
                    break;
                }
            }
        }

        children.0 = new_children;
    }
}

pub fn set_default_radius(
    word_query: Query<(&Radius, &CircleChildren), (With<Word>, Without<Letter>)>,
    mut letter_query: Query<(&Parent, &Letter, &mut Radius), Without<Word>>,
) {
    for (letter_parent, letter, mut letter_radius) in letter_query.iter_mut() {
        let (word_radius, word_children) = word_query.get(letter_parent.get()).unwrap();

        let number_of_letters = word_children.0.len() as f32;

        letter_radius.0 = match letter {
            Letter::Vocal(_) => (word_radius.0 * 0.75 * 0.4) / (1.0 + number_of_letters / 2.0),
            Letter::Consonant(_) => (word_radius.0 * 0.75) / (1.0 + number_of_letters / 2.0),
        };
    }
}

pub fn set_default_position(
    word_query: Query<(&Radius, &CircleChildren), (With<Word>, Without<Letter>)>,
    mut letter_query: Query<
        (&Parent, &Letter, &mut PositionData, &Radius, &SiblingIndex),
        Without<Word>,
    >,
) {
    for (letter_parent, letter, mut position_data, letter_radius, letter_index) in
        letter_query.iter_mut()
    {
        let (word_radius, word_children) = word_query.get(letter_parent.get()).unwrap();

        let number_of_letters = word_children.0.len();

        match letter {
            Letter::Vocal(vocal) => {
                position_data.distance = match VocalPlacement::from(*vocal) {
                    VocalPlacement::OnLine => word_radius.0,
                    VocalPlacement::Outside => word_radius.0 + letter_radius.0 * 1.5,
                    VocalPlacement::Inside => {
                        if number_of_letters > 1 {
                            word_radius.0 - letter_radius.0 * 1.5
                        } else {
                            0.0
                        }
                    }
                };
            }
            Letter::Consonant(consonant) => {
                position_data.distance = match ConsonantPlacement::from(*consonant) {
                    ConsonantPlacement::DeepCut => word_radius.0 - letter_radius.0 * 0.75,
                    ConsonantPlacement::Inside => {
                        if number_of_letters > 1 {
                            word_radius.0 - letter_radius.0 * 1.5
                        } else {
                            0.0
                        }
                    }
                    ConsonantPlacement::ShallowCut => word_radius.0,
                    ConsonantPlacement::OnLine => word_radius.0,
                };
            }
        }

        position_data.angle = Degree(letter_index.0 as f32 * (360.0 / number_of_letters as f32));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_split_lower_case_word() {
        let result: Vec<LetterText> =
            split_word_to_letters("aeioubjtthphwhghchkshydlrzcqgnvquhpwxfmsng").collect();

        let expected = [
            LetterText::new("a", Vocal::A),
            LetterText::new("e", Vocal::E),
            LetterText::new("i", Vocal::I),
            LetterText::new("o", Vocal::O),
            LetterText::new("u", Vocal::U),
            LetterText::new("b", Consonant::B),
            LetterText::new("j", Consonant::J),
            LetterText::new("t", Consonant::T),
            LetterText::new("th", Digraph::TH),
            LetterText::new("ph", Digraph::PH),
            LetterText::new("wh", Digraph::WH),
            LetterText::new("gh", Digraph::GH),
            LetterText::new("ch", Digraph::CH),
            LetterText::new("k", Consonant::K),
            LetterText::new("sh", Digraph::SH),
            LetterText::new("y", Consonant::Y),
            LetterText::new("d", Consonant::D),
            LetterText::new("l", Consonant::L),
            LetterText::new("r", Consonant::R),
            LetterText::new("z", Consonant::Z),
            LetterText::new("c", Consonant::C),
            LetterText::new("q", Consonant::Q),
            LetterText::new("g", Consonant::G),
            LetterText::new("n", Consonant::N),
            LetterText::new("v", Consonant::V),
            LetterText::new("qu", Digraph::QU),
            LetterText::new("h", Consonant::H),
            LetterText::new("p", Consonant::P),
            LetterText::new("w", Consonant::W),
            LetterText::new("x", Consonant::X),
            LetterText::new("f", Consonant::F),
            LetterText::new("m", Consonant::M),
            LetterText::new("s", Consonant::S),
            LetterText::new("ng", Digraph::NG),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn should_split_upper_case_word() {
        let result: Vec<LetterText> =
            split_word_to_letters("AEIOUBJTTHPHWHGHCHKSHYDLRZCQGNVQUHPWXFMSNG").collect();

        let expected = [
            LetterText::new("A", Vocal::A),
            LetterText::new("E", Vocal::E),
            LetterText::new("I", Vocal::I),
            LetterText::new("O", Vocal::O),
            LetterText::new("U", Vocal::U),
            LetterText::new("B", Consonant::B),
            LetterText::new("J", Consonant::J),
            LetterText::new("T", Consonant::T),
            LetterText::new("TH", Digraph::TH),
            LetterText::new("PH", Digraph::PH),
            LetterText::new("WH", Digraph::WH),
            LetterText::new("GH", Digraph::GH),
            LetterText::new("CH", Digraph::CH),
            LetterText::new("K", Consonant::K),
            LetterText::new("SH", Digraph::SH),
            LetterText::new("Y", Consonant::Y),
            LetterText::new("D", Consonant::D),
            LetterText::new("L", Consonant::L),
            LetterText::new("R", Consonant::R),
            LetterText::new("Z", Consonant::Z),
            LetterText::new("C", Consonant::C),
            LetterText::new("Q", Consonant::Q),
            LetterText::new("G", Consonant::G),
            LetterText::new("N", Consonant::N),
            LetterText::new("V", Consonant::V),
            LetterText::new("QU", Digraph::QU),
            LetterText::new("H", Consonant::H),
            LetterText::new("P", Consonant::P),
            LetterText::new("W", Consonant::W),
            LetterText::new("X", Consonant::X),
            LetterText::new("F", Consonant::F),
            LetterText::new("M", Consonant::M),
            LetterText::new("S", Consonant::S),
            LetterText::new("NG", Digraph::NG),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn should_split_mixed_case_digraph_letters() {
        let result: Vec<LetterText> =
            split_word_to_letters("tHThpHPhwHWhgHGhcHChsHShqUQunGNg").collect();

        let expected = [
            LetterText::new("tH", Digraph::TH),
            LetterText::new("Th", Digraph::TH),
            LetterText::new("pH", Digraph::PH),
            LetterText::new("Ph", Digraph::PH),
            LetterText::new("wH", Digraph::WH),
            LetterText::new("Wh", Digraph::WH),
            LetterText::new("gH", Digraph::GH),
            LetterText::new("Gh", Digraph::GH),
            LetterText::new("cH", Digraph::CH),
            LetterText::new("Ch", Digraph::CH),
            LetterText::new("sH", Digraph::SH),
            LetterText::new("Sh", Digraph::SH),
            LetterText::new("qU", Digraph::QU),
            LetterText::new("Qu", Digraph::QU),
            LetterText::new("nG", Digraph::NG),
            LetterText::new("Ng", Digraph::NG),
        ];

        assert_eq!(result, expected);
    }
}
