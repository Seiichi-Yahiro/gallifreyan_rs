use bevy::prelude::*;

use consonant::ConsonantDecoration;
use vocal::VocalDecoration;

use super::components::{Text, *};
use super::split_word_to_chars;

pub mod consonant;
pub mod vocal;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component)]
pub enum Letter {
    Vocal(Vocal),
    Consonant(Consonant),
}

impl TryFrom<&str> for Letter {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Vocal::try_from(value)
            .map(Self::Vocal)
            .or_else(|_| Consonant::try_from(value).map(Self::Consonant))
            .map_err(|_| {
                format!(
                    "Cannot assign letter to '{}' as it is not a valid letter!",
                    value
                )
            })
    }
}

#[derive(Bundle)]
pub struct LetterBundle {
    pub name: Name,
    pub letter: Letter,
    pub text: Text,
    pub dots: CircleChildren,
    pub line_slots: LineSlotChildren,
}

impl LetterBundle {
    pub fn new(text: String, letter: Letter) -> Self {
        Self {
            name: Name::new("Letter"),
            letter,
            text: Text(text),
            dots: Default::default(),
            line_slots: Default::default(),
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

pub fn convert_letters(
    mut commands: Commands,
    mut word_query: Query<(Entity, &Text, &mut CircleChildren), (With<Word>, Changed<Text>)>,
    mut letter_query: Query<(Entity, &mut Text, &mut Letter), Without<Word>>,
) {
    for (word_entity, word_text, mut children) in word_query.iter_mut() {
        let mut existing_letters = letter_query.iter_many_mut(children.iter());

        let mut new_letters_iter = split_word_to_chars(word_text).map(|it| {
            let letter = Letter::try_from(it).unwrap();
            (it.to_string(), letter)
        });

        let mut new_children: Vec<Entity> = Vec::new();

        loop {
            let next_existing_letter = existing_letters.fetch_next();
            let next_new_letter = new_letters_iter.next();

            match (next_existing_letter, next_new_letter) {
                // update letter
                (Some((letter_entity, mut text, mut letter)), Some((new_text, new_letter))) => {
                    if **text != new_text {
                        debug!("Update letter: {:?} -> {:?}", *letter, new_letter);

                        **text = new_text;
                        *letter = new_letter;
                    }

                    new_children.push(letter_entity);
                }
                // remove letter
                (Some((letter_entity, _text, letter)), None) => {
                    debug!("Despawn letter: {:?}", *letter);
                    commands.entity(letter_entity).despawn_recursive();
                }
                // add letter
                (None, Some((text, new_letter))) => {
                    debug!("Spawn letter: {:?}", new_letter);

                    let bundle = LetterBundle::new(text, new_letter);

                    let letter_entity = commands.spawn(bundle).id();
                    commands.entity(word_entity).add_child(letter_entity);
                    new_children.push(letter_entity);
                }
                (None, None) => {
                    break;
                }
            }
        }

        **children = new_children;
    }
}

#[cfg(test)]
mod test {
    use crate::plugins::text_converter::{
        test::test_component_update, SetText, TextConverterPlugin,
    };

    use super::*;

    fn test_count_letter_entities(
        text: &str,
        expected_letters: usize,
        expected_dots: usize,
        expected_line_slots: usize,
    ) {
        let mut app = App::new();
        app.add_plugins(TextConverterPlugin);

        app.world_mut()
            .resource_mut::<Events<SetText>>()
            .send(SetText(text.to_string()));

        app.update();

        let letters = app
            .world_mut()
            .query_filtered::<Entity, With<Letter>>()
            .iter(&app.world())
            .len();

        assert_eq!(letters, expected_letters);

        let dots = app
            .world_mut()
            .query_filtered::<Entity, With<Dot>>()
            .iter(&app.world())
            .len();

        assert_eq!(dots, expected_dots);

        let line_slots = app
            .world_mut()
            .query_filtered::<Entity, With<LineSlot>>()
            .iter(&app.world())
            .len();

        assert_eq!(line_slots, expected_line_slots);
    }

    #[test]
    fn should_spawn_b() {
        test_count_letter_entities("b", 1, 0, 0);
    }

    #[test]
    fn should_spawn_j() {
        test_count_letter_entities("j", 1, 0, 0);
    }

    #[test]
    fn should_spawn_t() {
        test_count_letter_entities("t", 1, 0, 0);
    }

    #[test]
    fn should_spawn_th() {
        test_count_letter_entities("th", 1, 0, 0);
    }

    #[test]
    fn should_spawn_ph() {
        test_count_letter_entities("ph", 1, 1, 0);
    }

    #[test]
    fn should_spawn_wh() {
        test_count_letter_entities("wh", 1, 1, 0);
    }

    #[test]
    fn should_spawn_gh() {
        test_count_letter_entities("gh", 1, 1, 0);
    }

    #[test]
    fn should_spawn_ch() {
        test_count_letter_entities("ch", 1, 2, 0);
    }

    #[test]
    fn should_spawn_k() {
        test_count_letter_entities("k", 1, 2, 0);
    }

    #[test]
    fn should_spawn_sh() {
        test_count_letter_entities("sh", 1, 2, 0);
    }

    #[test]
    fn should_spawn_y() {
        test_count_letter_entities("y", 1, 2, 0);
    }

    #[test]
    fn should_spawn_d() {
        test_count_letter_entities("d", 1, 3, 0);
    }

    #[test]
    fn should_spawn_l() {
        test_count_letter_entities("l", 1, 3, 0);
    }

    #[test]
    fn should_spawn_r() {
        test_count_letter_entities("r", 1, 3, 0);
    }

    #[test]
    fn should_spawn_z() {
        test_count_letter_entities("z", 1, 3, 0);
    }

    #[test]
    fn should_spawn_c() {
        test_count_letter_entities("c", 1, 4, 0);
    }

    #[test]
    fn should_spawn_q() {
        test_count_letter_entities("q", 1, 4, 0);
    }

    #[test]
    fn should_spawn_g() {
        test_count_letter_entities("g", 1, 0, 1);
    }

    #[test]
    fn should_spawn_n() {
        test_count_letter_entities("n", 1, 0, 1);
    }

    #[test]
    fn should_spawn_v() {
        test_count_letter_entities("v", 1, 0, 1);
    }

    #[test]
    fn should_spawn_qu() {
        test_count_letter_entities("qu", 1, 0, 1);
    }

    #[test]
    fn should_spawn_h() {
        test_count_letter_entities("h", 1, 0, 2);
    }

    #[test]
    fn should_spawn_p() {
        test_count_letter_entities("p", 1, 0, 2);
    }

    #[test]
    fn should_spawn_w() {
        test_count_letter_entities("w", 1, 0, 2);
    }

    #[test]
    fn should_spawn_x() {
        test_count_letter_entities("x", 1, 0, 2);
    }

    #[test]
    fn should_spawn_f() {
        test_count_letter_entities("f", 1, 0, 3);
    }

    #[test]
    fn should_spawn_m() {
        test_count_letter_entities("m", 1, 0, 3);
    }

    #[test]
    fn should_spawn_s() {
        test_count_letter_entities("s", 1, 0, 3);
    }

    #[test]
    fn should_spawn_ng() {
        test_count_letter_entities("ng", 1, 0, 3);
    }

    #[test]
    fn should_spawn_a() {
        test_count_letter_entities("a", 1, 0, 0);
    }

    #[test]
    fn should_spawn_e() {
        test_count_letter_entities("e", 1, 0, 0);
    }

    #[test]
    fn should_spawn_i() {
        test_count_letter_entities("i", 1, 0, 1);
    }

    #[test]
    fn should_spawn_o() {
        test_count_letter_entities("o", 1, 0, 0);
    }

    #[test]
    fn should_spawn_u() {
        test_count_letter_entities("u", 1, 0, 1);
    }

    #[test]
    fn should_update_letter_text() {
        test_component_update::<Text, Letter>("test", "text", |_before, after| {
            assert_eq!(*after[0], "t");
            assert_eq!(*after[1], "e");
            assert_eq!(*after[2], "x");
            assert_eq!(*after[3], "t");
        });
    }

    #[test]
    fn should_despawn_children() {
        let mut app = App::new();
        app.add_plugins(TextConverterPlugin);

        let assert_occurrences = |app: &mut App, line_slots: usize, dots: usize, letters: usize| {
            let line_slots_result = app
                .world_mut()
                .query_filtered::<Entity, With<LineSlot>>()
                .iter(&app.world())
                .len();
            let dots_result = app
                .world_mut()
                .query_filtered::<Entity, With<Dot>>()
                .iter(&app.world())
                .len();
            let letters_result = app
                .world_mut()
                .query_filtered::<Entity, With<Letter>>()
                .iter(&app.world())
                .len();

            assert_eq!(line_slots_result, line_slots);
            assert_eq!(dots_result, dots);
            assert_eq!(letters_result, letters);
        };

        app.world_mut()
            .resource_mut::<Events<SetText>>()
            .send(SetText("bdf".to_string()));

        app.update();

        assert_occurrences(&mut app, 3, 3, 3);

        app.world_mut()
            .resource_mut::<Events<SetText>>()
            .send(SetText("bd".to_string()));

        app.update();

        assert_occurrences(&mut app, 0, 3, 2);

        app.world_mut()
            .resource_mut::<Events<SetText>>()
            .send(SetText("b".to_string()));

        app.update();

        assert_occurrences(&mut app, 0, 0, 1);
    }
}
