use crate::image_types::{Text, *};
use crate::sidebar::text_converter::split_word_to_chars;
use bevy::prelude::*;

const NESTED_LETTER_TEXT_DELIMITER: &str = "~";

fn create_letters_from_word(
    word_text: &str,
    nesting_settings: &NestingSettings,
) -> Vec<(String, Letter)> {
    let letters = split_word_to_chars(word_text).map(|it| {
        let letter = Letter::try_from(it).unwrap();
        (it.to_string(), letter)
    });

    match nesting_settings {
        NestingSettings::None => letters.collect(),
        nesting_settings => letters.fold(Vec::new(), |mut acc, (text, letter)| {
            match letter {
                Letter::Vocal(vocal) => {
                    if let Some((previous_text, previous_letter)) = acc.pop() {
                        match previous_letter {
                            Letter::Consonant(consonant)
                                if nesting_settings.can_nest(consonant, vocal) =>
                            {
                                acc.push((
                                    previous_text + NESTED_LETTER_TEXT_DELIMITER + &text,
                                    Letter::ConsonantWithVocal { consonant, vocal },
                                ));
                            }
                            _ => {
                                acc.push((previous_text, previous_letter));
                                acc.push((text, letter));
                            }
                        }
                    } else {
                        acc.push((text, letter));
                    }
                }
                Letter::Consonant(_) | Letter::ConsonantWithVocal { .. } => {
                    acc.push((text, letter));
                }
            }

            acc
        }),
    }
}

pub fn convert_letters(
    mut commands: Commands,
    mut word_query: Query<
        (Entity, &Text, &Radius, &mut CircleChildren),
        (With<Word>, Changed<Text>),
    >,
    mut letter_query: Query<
        (
            Entity,
            &mut Text,
            &mut Letter,
            &mut Radius,
            &mut PositionData,
        ),
        (Without<Word>, Without<NestedVocal>),
    >,
    nesting_settings: Res<NestingSettings>,
) {
    for (word_entity, word_text, Radius(word_radius), mut children) in word_query.iter_mut() {
        let mut existing_letters = letter_query.iter_many_mut(children.iter());

        let new_letters = create_letters_from_word(word_text, &nesting_settings);

        let number_of_letters = new_letters.len();
        let mut new_letters_iter = new_letters.into_iter();

        let mut new_children: Vec<Entity> = Vec::with_capacity(number_of_letters);

        loop {
            let next_existing_letter = existing_letters.fetch_next();
            let next_new_letter = new_letters_iter.next();

            match (next_existing_letter, next_new_letter) {
                // update letter
                (
                    Some((letter_entity, mut text, mut letter, mut radius, mut position_data)),
                    Some((new_text, new_letter)),
                ) => {
                    let new_radius = new_letter.radius(*word_radius, number_of_letters);
                    let new_position_data = new_letter.position_data(
                        *word_radius,
                        number_of_letters,
                        new_children.len(),
                    );

                    **text = new_text;
                    *letter = new_letter;

                    if **radius != new_radius {
                        **radius = new_radius;
                    }

                    if *position_data != new_position_data {
                        *position_data = new_position_data;
                    }

                    new_children.push(letter_entity);
                }
                // remove letter
                (Some((letter_entity, _text, _letter, _radius, _position_data)), None) => {
                    commands.entity(letter_entity).despawn_recursive();
                }
                // add letter
                (None, Some((text, new_letter))) => {
                    let letter_bundle = LetterBundle::new(
                        text,
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

pub fn convert_nested_letters(
    mut commands: Commands,
    word_query: Query<&Radius, (With<Word>, Without<NestedVocal>)>,
    mut letter_query: Query<
        (
            Entity,
            &Parent,
            &Text,
            &Letter,
            &PositionData,
            &Radius,
            &mut NestedLetter,
        ),
        (Changed<Letter>, Without<NestedVocal>),
    >,
    position_correction_query: Query<Entity, With<NestedVocalPositionCorrection>>,
    mut nested_vocal_query: Query<
        (
            &Parent,
            &mut Text,
            &mut Letter,
            &mut Radius,
            &mut PositionData,
        ),
        With<NestedVocal>,
    >,
) {
    for (
        letter_entity,
        letter_parent,
        letter_text,
        letter,
        letter_position_data,
        letter_radius,
        mut nested,
    ) in letter_query.iter_mut()
    {
        match letter {
            Letter::ConsonantWithVocal { vocal, consonant } => {
                let word_radius = **word_query.get(letter_parent.get()).unwrap();
                let new_nested_text = letter_text
                    .split_once(NESTED_LETTER_TEXT_DELIMITER)
                    .unwrap()
                    .1
                    .to_string();

                if let Some(nested_entity) = nested.take() {
                    // update nested
                    if let Ok((
                        nested_parent,
                        mut nested_text,
                        mut nested_letter,
                        mut nested_radius,
                        mut nested_position_data,
                    )) = nested_vocal_query.get_mut(nested_entity)
                    {
                        let old_placement = match *nested_letter {
                            Letter::Vocal(vocal) => VocalPlacement::from(vocal),
                            _ => unreachable!(),
                        };

                        let new_placement = VocalPlacement::from(*vocal);

                        match (old_placement, new_placement) {
                            // add position correction
                            (
                                VocalPlacement::OnLine | VocalPlacement::Inside,
                                VocalPlacement::Outside,
                            ) => {
                                commands
                                    .entity(letter_entity)
                                    .remove_children(&[nested_entity])
                                    .with_children(|child_builder| {
                                        child_builder
                                            .spawn(NestedVocalPositionCorrectionBundle::new(
                                                letter_position_data.distance,
                                            ))
                                            .add_child(nested_entity);
                                    });
                            }
                            // remove position correction
                            (
                                VocalPlacement::Outside,
                                VocalPlacement::OnLine | VocalPlacement::Inside,
                            ) => {
                                commands
                                    .entity(nested_parent.get())
                                    .remove_children(&[nested_entity])
                                    .despawn();
                                commands.entity(letter_entity).add_child(nested_entity);
                            }
                            _ => {}
                        }

                        **nested_text = new_nested_text;
                        *nested_letter = Letter::Vocal(*vocal);

                        let new_nested_radius = vocal.nested_radius(**letter_radius);
                        let new_nested_position_data = vocal.nested_position_data(
                            ConsonantPlacement::from(*consonant),
                            **letter_radius,
                            letter_position_data.distance,
                            word_radius,
                        );

                        if **nested_radius != new_nested_radius {
                            **nested_radius = new_nested_radius;
                        }

                        if *nested_position_data != new_nested_position_data {
                            *nested_position_data = new_nested_position_data;
                        }
                    }

                    **nested = Some(nested_entity);
                } else {
                    // spawn nested
                    let vocal_id = commands
                        .entity(letter_entity)
                        .add_children(|child_builder| {
                            let vocal_bundle = NestedVocalBundle::new(
                                new_nested_text,
                                *vocal,
                                ConsonantPlacement::from(*consonant),
                                **letter_radius,
                                letter_position_data.distance,
                                word_radius,
                            );

                            if VocalPlacement::Outside == VocalPlacement::from(*vocal) {
                                child_builder
                                    .spawn(NestedVocalPositionCorrectionBundle::new(
                                        letter_position_data.distance,
                                    ))
                                    .add_children(|child_builder| {
                                        child_builder.spawn(vocal_bundle).id()
                                    })
                            } else {
                                child_builder.spawn(vocal_bundle).id()
                            }
                        });

                    **nested = Some(vocal_id);
                }
            }
            Letter::Consonant(_) | Letter::Vocal(_) => {
                // remove nested
                if let Some(nested_entity) = nested.take() {
                    let position_correction_entity = nested_vocal_query
                        .get(nested_entity)
                        .and_then(|(parent, _, _, _, _)| {
                            position_correction_query.get(parent.get())
                        });

                    if let Ok(position_correction_entity) = position_correction_entity {
                        commands
                            .entity(position_correction_entity)
                            .despawn_recursive();
                    } else {
                        commands.entity(nested_entity).despawn_recursive();
                    }
                }
            }
        }
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
    fn should_update_letter_text() {
        test_component_update::<Text, Letter>(
            "test",
            "text",
            NestingSettings::None,
            |_before, after| {
                assert_eq!(*after[0], "t");
                assert_eq!(*after[1], "e");
                assert_eq!(*after[2], "x");
                assert_eq!(*after[3], "t");
            },
        );
    }

    #[test]
    fn should_decrease_letter_radius() {
        test_component_update::<Radius, Letter>(
            "jj",
            "jjj",
            NestingSettings::None,
            |before, after| {
                assert!(before[1] > after[1]);
            },
        );
    }

    #[test]
    fn should_increase_letter_radius() {
        test_component_update::<Radius, Letter>(
            "jjj",
            "jj",
            NestingSettings::None,
            |before, after| {
                assert!(before[1] < after[1]);
            },
        );
    }

    #[test]
    fn should_not_update_letter_radius() {
        test_component_update::<Radius, Letter>(
            "jjj",
            "jjj",
            NestingSettings::None,
            |before, after| {
                assert_eq!(before[1], after[1]);
            },
        );
    }

    #[test]
    fn should_decrease_letter_angle() {
        test_component_update::<PositionData, Letter>(
            "jj",
            "jjj",
            NestingSettings::None,
            |before, after| {
                assert_eq!(after[0].angle.as_degrees(), 0.0);
                assert!(before[1].angle > after[1].angle);
                assert!(after[0].angle < after[1].angle && after[1].angle < after[2].angle);
            },
        );
    }

    #[test]
    fn should_increase_letter_angle() {
        test_component_update::<PositionData, Letter>(
            "jjj",
            "jj",
            NestingSettings::None,
            |before, after| {
                assert_eq!(after[0].angle.as_degrees(), 0.0);
                assert!(before[1].angle < after[1].angle);
                assert!(after[0].angle < after[1].angle);
            },
        );
    }

    #[test]
    fn should_not_update_letter_angle() {
        test_component_update::<PositionData, Letter>(
            "jjj",
            "jjj",
            NestingSettings::None,
            |before, after| {
                assert_eq!(before[1].angle, after[1].angle);
            },
        );
    }

    #[test]
    fn should_despawn_children() {
        let mut app = create_app();

        let assert_occurrences = |app: &mut App, line_slots: usize, dots: usize, letters: usize| {
            let line_slots_result = app
                .world
                .query_filtered::<Entity, With<LineSlot>>()
                .iter(&app.world)
                .len();
            let dots_result = app
                .world
                .query_filtered::<Entity, With<Dot>>()
                .iter(&app.world)
                .len();
            let letters_result = app
                .world
                .query_filtered::<Entity, With<Letter>>()
                .iter(&app.world)
                .len();

            assert_eq!(line_slots_result, line_slots);
            assert_eq!(dots_result, dots);
            assert_eq!(letters_result, letters);
        };

        app.world
            .resource_mut::<Events<SetText>>()
            .send(SetText("bdf".to_string()));

        app.update();

        assert_occurrences(&mut app, 3, 3, 3);

        app.world
            .resource_mut::<Events<SetText>>()
            .send(SetText("bd".to_string()));

        app.update();

        assert_occurrences(&mut app, 0, 3, 2);

        app.world
            .resource_mut::<Events<SetText>>()
            .send(SetText("b".to_string()));

        app.update();

        assert_occurrences(&mut app, 0, 0, 1);
    }

    #[test]
    fn should_nest_all_vocals() {
        let result = create_letters_from_word("bbabibubebo", &NestingSettings::All);
        let expected = [
            ("b".to_string(), Letter::Consonant(Consonant::B)),
            (
                "b~a".to_string(),
                Letter::ConsonantWithVocal {
                    consonant: Consonant::B,
                    vocal: Vocal::A,
                },
            ),
            (
                "b~i".to_string(),
                Letter::ConsonantWithVocal {
                    consonant: Consonant::B,
                    vocal: Vocal::I,
                },
            ),
            (
                "b~u".to_string(),
                Letter::ConsonantWithVocal {
                    consonant: Consonant::B,
                    vocal: Vocal::U,
                },
            ),
            (
                "b~e".to_string(),
                Letter::ConsonantWithVocal {
                    consonant: Consonant::B,
                    vocal: Vocal::E,
                },
            ),
            (
                "b~o".to_string(),
                Letter::ConsonantWithVocal {
                    consonant: Consonant::B,
                    vocal: Vocal::O,
                },
            ),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn should_nest_no_vocals() {
        let result = create_letters_from_word("bbabibubebo", &NestingSettings::None);
        let expected = [
            ("b".to_string(), Letter::Consonant(Consonant::B)),
            ("b".to_string(), Letter::Consonant(Consonant::B)),
            ("a".to_string(), Letter::Vocal(Vocal::A)),
            ("b".to_string(), Letter::Consonant(Consonant::B)),
            ("i".to_string(), Letter::Vocal(Vocal::I)),
            ("b".to_string(), Letter::Consonant(Consonant::B)),
            ("u".to_string(), Letter::Vocal(Vocal::U)),
            ("b".to_string(), Letter::Consonant(Consonant::B)),
            ("e".to_string(), Letter::Vocal(Vocal::E)),
            ("b".to_string(), Letter::Consonant(Consonant::B)),
            ("o".to_string(), Letter::Vocal(Vocal::O)),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn should_nest_custom_vocals() {
        let rules = [(Consonant::B, Vocal::A), (Consonant::B, Vocal::O)]
            .into_iter()
            .collect();

        let result = create_letters_from_word("bbabibubebo", &NestingSettings::Custom(rules));
        let expected = [
            ("b".to_string(), Letter::Consonant(Consonant::B)),
            (
                "b~a".to_string(),
                Letter::ConsonantWithVocal {
                    consonant: Consonant::B,
                    vocal: Vocal::A,
                },
            ),
            ("b".to_string(), Letter::Consonant(Consonant::B)),
            ("i".to_string(), Letter::Vocal(Vocal::I)),
            ("b".to_string(), Letter::Consonant(Consonant::B)),
            ("u".to_string(), Letter::Vocal(Vocal::U)),
            ("b".to_string(), Letter::Consonant(Consonant::B)),
            ("e".to_string(), Letter::Vocal(Vocal::E)),
            (
                "b~o".to_string(),
                Letter::ConsonantWithVocal {
                    consonant: Consonant::B,
                    vocal: Vocal::O,
                },
            ),
        ];

        assert_eq!(result, expected);
    }

    fn assert_spawn_nested(consonant: &str, vocal: &str) {
        let mut app = create_app();
        app.insert_resource(NestingSettings::All);

        app.world
            .resource_mut::<Events<SetText>>()
            .send(SetText(consonant.to_string() + vocal));

        app.update();

        let (consonant_entity, consonant_text, consonant_nested_letter) = {
            let consonant=  app.world.query_filtered::<(Entity, &Text, &NestedLetter), (With<Letter>, Without<NestedVocal>)>().single(&app.world);
            (consonant.0, consonant.1.clone(), *consonant.2)
        };

        let (vocal_entity, vocal_text, vocal_nested_letter, vocal_parent) = {
            let vocal = app.world.query_filtered::<(Entity, &Text, &NestedLetter, &Parent), (With<Letter>, With<NestedVocal>)>().single(&app.world);
            (vocal.0, vocal.1.clone(), *vocal.2, vocal.3.get())
        };

        assert_eq!(*consonant_text, format!("{}~{}", consonant, vocal));
        assert_eq!(*vocal_text, vocal);

        assert_eq!(consonant_nested_letter.0, Some(vocal_entity));
        assert_eq!(vocal_parent, consonant_entity);

        assert_eq!(vocal_nested_letter.0, None);
    }

    #[test]
    fn spawn_nested_e() {
        assert_spawn_nested("b", "e");
    }

    #[test]
    fn spawn_nested_i() {
        assert_spawn_nested("b", "i");
    }

    #[test]
    fn spawn_nested_o() {
        assert_spawn_nested("b", "o");
    }

    #[test]
    fn spawn_nested_u() {
        assert_spawn_nested("b", "u");
    }

    #[test]
    fn spawn_nested_a() {
        let mut app = create_app();
        app.insert_resource(NestingSettings::All);

        app.world
            .resource_mut::<Events<SetText>>()
            .send(SetText("ba".to_string()));

        app.update();

        let (consonant_entity, consonant_text, consonant_nested_letter) = {
            let consonant=  app.world.query_filtered::<(Entity, &Text, &NestedLetter), (With<Letter>, Without<NestedVocal>)>().single(&app.world);
            (consonant.0, consonant.1.clone(), *consonant.2)
        };

        let (vocal_entity, vocal_text, vocal_nested_letter, vocal_parent) = {
            let vocal = app.world.query_filtered::<(Entity, &Text, &NestedLetter, &Parent), (With<Letter>, With<NestedVocal>)>().single(&app.world);
            (vocal.0, vocal.1.clone(), *vocal.2, vocal.3.get())
        };

        let (vocal_position_correction_entity, vocal_position_correction_parent) = app
            .world
            .query_filtered::<(Entity, &Parent), With<NestedVocalPositionCorrection>>()
            .single(&app.world);

        assert_eq!(*consonant_text, "b~a");
        assert_eq!(*vocal_text, "a");

        assert_eq!(consonant_nested_letter.0, Some(vocal_entity));
        assert_eq!(vocal_parent, vocal_position_correction_entity);
        assert_eq!(vocal_position_correction_parent.get(), consonant_entity);

        assert_eq!(vocal_nested_letter.0, None);
    }

    #[test]
    fn should_update_nested_letter_text() {
        test_component_update::<Text, Letter>(
            "be",
            "bi",
            NestingSettings::All,
            |_before, after| {
                assert_eq!(*after[0], "b~i");
            },
        );
    }

    #[test]
    fn should_remove_nested_letter_text() {
        test_component_update::<Text, Letter>("be", "b", NestingSettings::All, |_before, after| {
            assert_eq!(*after[0], "b");
        });
    }

    #[test]
    fn should_despawn_nested_children() {
        let mut app = create_app();
        app.insert_resource(NestingSettings::All);

        let assert_occurrences = |app: &mut App,
                                  line_slots: usize,
                                  letters: usize,
                                  nested_letters: usize,
                                  position_correction: usize| {
            let line_slots_result = app
                .world
                .query_filtered::<Entity, With<LineSlot>>()
                .iter(&app.world)
                .len();
            let letters_result = app
                .world
                .query_filtered::<Entity, With<Letter>>()
                .iter(&app.world)
                .len();
            let nested_letters_result = app
                .world
                .query_filtered::<Entity, (With<Letter>, With<NestedVocal>)>()
                .iter(&app.world)
                .len();
            let position_correction_result = app
                .world
                .query_filtered::<Entity, With<NestedVocalPositionCorrection>>()
                .iter(&app.world)
                .len();

            assert_eq!(line_slots_result, line_slots);
            assert_eq!(letters_result, letters);
            assert_eq!(nested_letters_result, nested_letters);
            assert_eq!(position_correction_result, position_correction);
        };

        app.world
            .resource_mut::<Events<SetText>>()
            .send(SetText("bbabi".to_string()));

        app.update();

        assert_occurrences(&mut app, 1, 5, 2, 1);

        app.world
            .resource_mut::<Events<SetText>>()
            .send(SetText("bba".to_string()));

        app.update();

        assert_occurrences(&mut app, 0, 3, 1, 1);

        app.world
            .resource_mut::<Events<SetText>>()
            .send(SetText("b".to_string()));

        app.update();

        assert_occurrences(&mut app, 0, 1, 0, 0);
    }

    #[test]
    fn should_remove_position_correction_on_update() {
        test_component_update::<PositionData, NestedVocalPositionCorrection>(
            "ba",
            "be",
            NestingSettings::All,
            |_before, after| {
                assert_eq!(after.len(), 0);
            },
        );
    }

    #[test]
    fn should_add_position_correction_on_update() {
        test_component_update::<PositionData, NestedVocalPositionCorrection>(
            "be",
            "ba",
            NestingSettings::All,
            |_before, after| {
                assert_eq!(after.len(), 1);
            },
        );
    }
}
