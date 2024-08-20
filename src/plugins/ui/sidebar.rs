use crate::plugins::text_converter::letter::{Decorated, Letter};
use crate::plugins::text_converter::{SentenceText, SetText, SetTextSet, TextModification};
use crate::plugins::ui::icons::Icons;
use crate::plugins::ui::widgets;
use crate::plugins::ui::widgets::foldable::{Foldable, FoldableLabel};
use crate::plugins::ui::{styles, UiRoot};
use bevy::prelude::*;
use bevy::utils::HashSet;
use itertools::Itertools;
use std::collections::BTreeMap;

pub struct SidebarPlugin;

impl Plugin for SidebarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SentenceEntity>()
            .add_systems(Startup, setup.in_set(UiSidebarSet))
            .add_systems(
                Update,
                (
                    spawn_sentences,
                    spawn_words,
                    handle_text_modifications,
                    despawn_words,
                    despawn_sentences,
                    update_foldable_header_texts,
                )
                    .chain()
                    .after(SetTextSet)
                    .run_if(on_event::<TextModification>()),
            );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct UiSidebarSet;

#[derive(Component)]
pub struct Sidebar {
    tree_content: Entity,
}

fn setup(mut commands: Commands, ui_root_query: Query<Entity, With<UiRoot>>) {
    let root = ui_root_query.get_single().unwrap();

    let text_input = widgets::text_input::create(&mut commands, "Sentence".to_string());
    commands.entity(text_input).observe(on_sentence_change);

    let (scroll_area, tree_content) = widgets::scroll_area::create(&mut commands);

    let tree_container = commands
        .spawn((
            Name::new("Tree Container"),
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    align_self: AlignSelf::Stretch,
                    overflow: Overflow {
                        x: OverflowAxis::Hidden,
                        y: OverflowAxis::Hidden,
                    },
                    ..default()
                },
                ..default()
            },
        ))
        .add_child(scroll_area)
        .id();

    commands
        .spawn((
            Name::new("Ui Sidebar"),
            Sidebar { tree_content },
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(styles::PADDING),
                    width: Val::Percent(20.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(styles::PADDING)),
                    border: UiRect::right(Val::Px(styles::BORDER_SIZE)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                background_color: BackgroundColor(styles::BACKGROUND_COLOR),
                border_color: BorderColor(styles::BORDER_COLOR),
                ..default()
            },
        ))
        .set_parent(root)
        .push_children(&[text_input, tree_container]);
}

fn on_sentence_change(
    trigger: Trigger<widgets::text_input::Changed>,
    mut set_text_events: EventWriter<SetText>,
) {
    let text = trigger.event().0.clone();
    set_text_events.send(SetText(text));
}

#[derive(Debug, Default, Resource)]
struct SentenceEntity {
    entity: Option<Entity>,
    words: Vec<WordEntity>,
}

#[derive(Debug)]
struct WordEntity {
    entity: Entity,
    letters: BTreeMap<usize, LetterEntity>,
}

#[derive(Debug)]
struct LetterEntity {
    entity: Entity,
}

fn spawn_sentences(
    mut commands: Commands,
    sentence_text: Res<SentenceText>,
    mut sentence_entity: ResMut<SentenceEntity>,
    icons: Res<Icons>,
    side_bar_query: Query<&Sidebar>,
) {
    if sentence_text.text.is_empty() || sentence_entity.entity.is_some() {
        return;
    }

    let (foldable, _content) =
        widgets::foldable::create(&mut commands, &icons, sentence_text.text.clone(), true);

    sentence_entity.entity = Some(foldable);

    let sidebar = side_bar_query.get_single().unwrap();

    commands.entity(sidebar.tree_content).add_child(foldable);
}

fn despawn_sentences(
    mut commands: Commands,
    sentence_text: Res<SentenceText>,
    mut sentence_entity: ResMut<SentenceEntity>,
) {
    if !sentence_text.text.is_empty() || sentence_entity.entity.is_none() {
        return;
    }

    commands
        .entity(sentence_entity.entity.take().unwrap())
        .despawn_recursive();

    sentence_entity.words = Vec::new();
}

fn spawn_words(
    mut commands: Commands,
    sentence_text: Res<SentenceText>,
    mut sentence_entity: ResMut<SentenceEntity>,
    foldable_query: Query<&Foldable>,
    icons: Res<Icons>,
) {
    if sentence_entity.words.len() >= sentence_text.words.len() {
        return;
    }

    let word_children = sentence_text.words[sentence_entity.words.len()..]
        .iter()
        .map(|word| {
            let (foldable, _content) =
                widgets::foldable::create(&mut commands, &icons, word.text.clone(), true);

            foldable
        })
        .collect_vec();

    let sentence_foldable_content = foldable_query
        .get(
            sentence_entity
                .entity
                .expect("Sentence should have been created"),
        )
        .unwrap()
        .content;

    commands
        .entity(sentence_foldable_content)
        .push_children(&word_children);

    sentence_entity
        .words
        .extend(word_children.into_iter().map(|word| WordEntity {
            entity: word,
            letters: BTreeMap::new(),
        }));
}

fn despawn_words(
    mut commands: Commands,
    sentence_text: Res<SentenceText>,
    mut sentence_entity: ResMut<SentenceEntity>,
) {
    if sentence_entity.words.len() < sentence_text.words.len() {
        return;
    }

    for word_entity in sentence_entity.words.drain(sentence_text.words.len()..) {
        commands.entity(word_entity.entity).despawn_recursive();
    }
}

fn handle_text_modifications(
    mut commands: Commands,
    mut text_modification_events: EventReader<TextModification>,
    mut sentence_entity: ResMut<SentenceEntity>,
    foldable_query: Query<&Foldable>,
    icons: Res<Icons>,
) {
    let mut insert_tasks = Vec::new();

    // first remove all old letters
    for event in text_modification_events.read() {
        match event {
            TextModification::Create { new_id, text } => {
                let letter = Letter::try_from(text.as_str()).unwrap();

                let entity = if letter.dots() + letter.lines() == 0 {
                    commands
                        .spawn(TextBundle::from_section(text, styles::TEXT_STYLE))
                        .id()
                } else {
                    let (foldable, content) =
                        widgets::foldable::create(&mut commands, &icons, text.clone(), true);

                    for _ in 0..letter.dots() {
                        commands
                            .spawn(TextBundle::from_section("DOT", styles::TEXT_STYLE))
                            .set_parent(content);
                    }

                    for _ in 0..letter.lines() {
                        commands
                            .spawn(TextBundle::from_section("LINE", styles::TEXT_STYLE))
                            .set_parent(content);
                    }

                    foldable
                };

                insert_tasks.push((*new_id, LetterEntity { entity }));
            }
            TextModification::Move { old_id, new_id, .. } => {
                let letter_entity = sentence_entity.words[old_id.word]
                    .letters
                    .remove(&old_id.letter)
                    .unwrap();

                insert_tasks.push((*new_id, letter_entity));
            }
            TextModification::Delete { old_id, .. } => {
                let letter_entity = sentence_entity.words[old_id.word]
                    .letters
                    .remove(&old_id.letter)
                    .unwrap();

                commands.entity(letter_entity.entity).despawn_recursive();
            }
        }
    }

    // then insert the letters at the correct location
    for (new_id, letter_entity) in insert_tasks {
        let word = &mut sentence_entity.words[new_id.word];
        let letters = &mut word.letters;

        let word_foldable_content = foldable_query.get(word.entity).unwrap().content;

        if new_id.letter >= letters.len() {
            commands
                .entity(word_foldable_content)
                .add_child(letter_entity.entity);
        } else {
            commands
                .entity(word_foldable_content)
                .insert_children(new_id.letter, &[letter_entity.entity]);
        }

        letters.insert(new_id.letter, letter_entity);
    }
}

fn update_foldable_header_texts(
    mut text_modification_events: EventReader<TextModification>,
    sentence_text: Res<SentenceText>,
    sentence_entity: Res<SentenceEntity>,
    foldable_query: Query<&Foldable>,
    mut label_query: Query<&mut Text, With<FoldableLabel>>,
) {
    let words = text_modification_events
        .read()
        .flat_map(|modification| match modification {
            TextModification::Create { new_id, .. } => {
                vec![new_id]
            }
            TextModification::Move { old_id, new_id, .. } => {
                vec![old_id, new_id]
            }
            TextModification::Delete { old_id, .. } => {
                vec![old_id]
            }
        })
        .fold(HashSet::new(), |mut words, letter_id| {
            words.insert((letter_id.sentence, letter_id.word));
            words
        });

    let sentence_chunks = words.iter().chunk_by(|(sentence, _)| *sentence);

    for (_sentence_index, words) in sentence_chunks.into_iter() {
        if let Some(entity) = sentence_entity.entity {
            let label_entity = foldable_query.get(entity).unwrap().label;
            let mut text = label_query.get_mut(label_entity).unwrap();
            text.sections[0].value = sentence_text.text.clone();
        }

        for (_sentence_index, word_index) in words {
            if let Some(word_entity) = sentence_entity.words.get(*word_index) {
                let label_entity = foldable_query.get(word_entity.entity).unwrap().label;
                let mut text = label_query.get_mut(label_entity).unwrap();
                text.sections[0].value = sentence_text.words[*word_index].text.clone();
            }
        }
    }
}
