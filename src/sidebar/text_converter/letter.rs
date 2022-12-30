use crate::image_types::{Text, *};
use bevy::prelude::*;

pub fn convert_letters(
    mut commands: Commands,
    mut word_query: Query<
        (Entity, &Text, &Radius, &mut CircleChildren),
        (With<Word>, Changed<Text>),
    >,
    mut letter_query: Query<
        (
            Entity,
            &mut Letter,
            &mut Text,
            &mut Radius,
            &mut PositionData,
        ),
        Without<Word>,
    >,
) {
    for (word_entity, word_text, Radius(word_radius), mut children) in word_query.iter_mut() {
        let mut existing_letters = letter_query.iter_many_mut(children.iter());

        let new_letters: Vec<String> = super::split_word_to_chars(word_text)
            .map(|it| it.to_string())
            .collect();

        let number_of_letters = new_letters.len();
        let mut new_letters_iter = new_letters.into_iter();

        let mut new_children: Vec<Entity> = Vec::with_capacity(number_of_letters);

        loop {
            let next_existing_letter = existing_letters.fetch_next();
            let next_new_letter = new_letters_iter.next();

            match (next_existing_letter, next_new_letter) {
                // update letter
                (
                    Some((
                        letter_entity,
                        mut letter,
                        mut letter_text,
                        mut radius,
                        mut position_data,
                    )),
                    Some(new_letter),
                ) => {
                    *letter = Letter::try_from(new_letter.as_str()).unwrap();

                    let new_radius = letter.radius(*word_radius, number_of_letters);
                    let new_position_data =
                        letter.position_data(*word_radius, number_of_letters, new_children.len());

                    // TODO text change
                    //if **letter_text != new_letter {
                    **letter_text = new_letter;
                    //}

                    if **radius != new_radius {
                        **radius = new_radius;
                    }

                    if *position_data != new_position_data {
                        *position_data = new_position_data;
                    }

                    new_children.push(letter_entity);
                }
                // remove letter
                (Some((letter_entity, _letter, _letter_text, _radius, _position_data)), None) => {
                    commands.entity(letter_entity).despawn_recursive();
                }
                // add letter
                (None, Some(new_letter)) => {
                    let letter = Letter::try_from(new_letter.as_str()).unwrap();

                    let letter_bundle = LetterBundle::new(
                        letter,
                        new_letter,
                        *word_radius,
                        number_of_letters,
                        new_children.len(),
                    );

                    let letter_entity = commands.spawn(letter_bundle).id();
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
    use super::super::test::{create_app, test_component_update};
    use super::super::SetText;
    use super::*;

    fn test_count_letter_entities(
        text: &str,
        expected_letters: usize,
        expected_dots: usize,
        expected_line_slots: usize,
    ) {
        let mut app = create_app();

        app.world
            .resource_mut::<Events<SetText>>()
            .send(SetText(text.to_string()));

        app.update();

        let letters = app
            .world
            .query_filtered::<Entity, With<Letter>>()
            .iter(&app.world)
            .len();

        assert_eq!(letters, expected_letters);

        let dots = app
            .world
            .query_filtered::<Entity, With<Dot>>()
            .iter(&app.world)
            .len();

        assert_eq!(dots, expected_dots);

        let line_slots = app
            .world
            .query_filtered::<Entity, With<LineSlot>>()
            .iter(&app.world)
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
    fn should_decrease_letter_radius() {
        test_component_update::<Radius, Letter>("jj", "jjj", |before, after| {
            assert!(before[1] > after[1]);
        });
    }

    #[test]
    fn should_increase_letter_radius() {
        test_component_update::<Radius, Letter>("jjj", "jj", |before, after| {
            assert!(before[1] < after[1]);
        });
    }

    #[test]
    fn should_not_update_letter_radius() {
        test_component_update::<Radius, Letter>("jjj", "jjj", |before, after| {
            assert_eq!(before[1], after[1]);
        });
    }

    #[test]
    fn should_decrease_letter_angle() {
        test_component_update::<PositionData, Letter>("jj", "jjj", |before, after| {
            assert_eq!(after[0].angle.as_degrees(), 0.0);
            assert!(before[1].angle > after[1].angle);
            assert!(after[0].angle < after[1].angle && after[1].angle < after[2].angle);
        });
    }

    #[test]
    fn should_increase_letter_angle() {
        test_component_update::<PositionData, Letter>("jjj", "jj", |before, after| {
            assert_eq!(after[0].angle.as_degrees(), 0.0);
            assert!(before[1].angle < after[1].angle);
            assert!(after[0].angle < after[1].angle);
        });
    }

    #[test]
    fn should_not_update_letter_angle() {
        test_component_update::<PositionData, Letter>("jjj", "jjj", |before, after| {
            assert_eq!(before[1].angle, after[1].angle);
        });
    }
}
