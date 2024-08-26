use crate::plugins::text_converter::prelude::*;
use crate::plugins::text_converter::TextConversionSet;
use crate::plugins::ui::icons::Icons;
use crate::plugins::ui::sidebar::{Sidebar, UiSidebarSet};
use crate::plugins::ui::widgets::foldable::Foldable;
use crate::plugins::ui::{styles, widgets};
use bevy::prelude::*;
use bevy::utils::HashMap;

pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup.after(super::setup).in_set(UiSidebarSet))
            .init_resource::<TextMap>()
            .configure_sets(
                Update,
                TreeUpdateSet
                    .in_set(SetSentenceSet)
                    .after(TextConversionSet),
            )
            .add_systems(
                Update,
                (
                    spawn_sentence_foldable,
                    spawn_word_foldable,
                    spawn_letter_foldable,
                    spawn_dot_text,
                    spawn_line_slot_text,
                    udpate_tree_entry_labels,
                )
                    .chain()
                    .in_set(TreeUpdateSet),
            )
            .observe(despawn_tree_entry::<Sentence>)
            .observe(despawn_tree_entry::<Word>)
            .observe(despawn_tree_entry::<Letter>)
            .observe(despawn_tree_entry::<Dot>)
            .observe(despawn_tree_entry::<LineSlot>);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct TreeUpdateSet;

#[derive(Component)]
pub struct Tree {
    content: Entity,
}

fn setup(mut commands: Commands, side_bar_query: Query<Entity, With<Sidebar>>) {
    let (scroll_area, tree_content) = widgets::scroll_area::create(&mut commands);

    let tree_container = commands
        .spawn((
            Name::new("Tree Container"),
            Tree {
                content: tree_content,
            },
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

    let sidebar = side_bar_query.get_single().unwrap();
    commands.entity(sidebar).add_child(tree_container);
}

#[derive(Debug, Default, Resource, Deref, DerefMut)]
struct TextMap(HashMap<Entity, Entity>);

fn spawn_sentence_foldable(
    mut commands: Commands,
    sentence_query: Query<(Entity, &GFText), Added<Sentence>>,
    icons: Res<Icons>,
    mut text_map: ResMut<TextMap>,
    tree_query: Query<&Tree>,
) {
    for (sentence_entity, text) in sentence_query.iter() {
        let (sentence_foldable, _content) =
            widgets::foldable::create(&mut commands, &icons, text.0.clone(), true);

        debug!("Spawn sentence: {}, {:?}", text.0, sentence_foldable);

        let tree = tree_query.get_single().unwrap();
        commands.entity(tree.content).add_child(sentence_foldable);

        text_map.insert(sentence_entity, sentence_foldable);
    }
}

fn spawn_word_foldable(
    mut commands: Commands,
    foldable_query: Query<&Foldable>,
    word_query: Query<(Entity, &GFText, &Parent, &SiblingIndex), Added<Word>>,
    icons: Res<Icons>,
    mut text_map: ResMut<TextMap>,
) {
    for (word_entity, text, parent, _sibling_index) in
        word_query.iter().sort_unstable::<&SiblingIndex>()
    {
        let (word_foldable, _content) =
            widgets::foldable::create(&mut commands, &icons, text.0.clone(), true);

        debug!("Spawn word: {}, {:?}", text.0, word_foldable);

        let sentence_foldable_entity = *text_map.get(&parent.get()).unwrap();
        let sentence_foldable = foldable_query.get(sentence_foldable_entity).unwrap();

        commands
            .entity(sentence_foldable.content)
            .add_child(word_foldable);

        text_map.insert(word_entity, word_foldable);
    }
}

fn spawn_letter_foldable(
    mut commands: Commands,
    foldable_query: Query<&Foldable>,
    letter_query: Query<(Entity, &GFText, &Parent, &SiblingIndex), Added<Letter>>,
    icons: Res<Icons>,
    mut text_map: ResMut<TextMap>,
) {
    for (letter_entity, text, parent, _sibling_index) in
        letter_query.iter().sort_unstable::<&SiblingIndex>()
    {
        let (letter_foldable, _content) =
            widgets::foldable::create(&mut commands, &icons, text.0.clone(), false);

        debug!("Spawn letter: {}, {:?}", text.0, letter_foldable);

        let word_foldable_entity = *text_map.get(&parent.get()).unwrap();
        let word_foldable = foldable_query.get(word_foldable_entity).unwrap();

        commands
            .entity(word_foldable.content)
            .add_child(letter_foldable);

        text_map.insert(letter_entity, letter_foldable);
    }
}

fn spawn_dot_text(
    mut commands: Commands,
    foldable_query: Query<&Foldable>,
    dot_query: Query<(Entity, &Parent, &SiblingIndex), Added<Dot>>,
    mut text_map: ResMut<TextMap>,
) {
    for (dot_entity, parent, _sibling_index) in dot_query.iter().sort_unstable::<&SiblingIndex>() {
        let letter_foldable_entity = *text_map.get(&parent.get()).unwrap();
        let letter_foldable = foldable_query.get(letter_foldable_entity).unwrap();

        let dot = commands
            .spawn(TextBundle::from_section("DOT", styles::TEXT_STYLE))
            .set_parent(letter_foldable.content)
            .id();

        debug!("Spawn dot: {:?}", dot);

        text_map.insert(dot_entity, dot);
    }
}

fn spawn_line_slot_text(
    mut commands: Commands,
    foldable_query: Query<&Foldable>,
    line_slot_query: Query<(Entity, &Parent, &SiblingIndex), Added<LineSlot>>,
    mut text_map: ResMut<TextMap>,
) {
    for (line_slot_entity, parent, _sibling_index) in
        line_slot_query.iter().sort_unstable::<&SiblingIndex>()
    {
        let letter_foldable_entity = *text_map.get(&parent.get()).unwrap();
        let letter_foldable = foldable_query.get(letter_foldable_entity).unwrap();

        let line_slot = commands
            .spawn(TextBundle::from_section("LINE", styles::TEXT_STYLE))
            .set_parent(letter_foldable.content)
            .id();

        debug!("Spawn line_slot: {:?}", line_slot);

        text_map.insert(line_slot_entity, line_slot);
    }
}

fn despawn_tree_entry<T: Component>(
    trigger: Trigger<OnRemove, T>,
    mut commands: Commands,
    mut text_map: ResMut<TextMap>,
) {
    let tree_entry = text_map
        .remove(&trigger.entity())
        .expect("Tree entry should have been created");

    debug!("Despawn: {:?}", tree_entry);

    commands.entity(tree_entry).despawn_recursive();
}

fn udpate_tree_entry_labels(
    text_query: Query<(Entity, Ref<GFText>)>,
    text_map: Res<TextMap>,
    foldable_query: Query<&Foldable>,
    mut label_query: Query<&mut Text>,
) {
    for (text_entity, text) in text_query.iter() {
        if text.is_added() || !text.is_changed() {
            continue;
        }

        let foldable_entity = *text_map.get(&text_entity).unwrap();
        debug!("Update label: {}, {:?}", text.0, foldable_entity);

        let label_entity = foldable_query.get(foldable_entity).unwrap().label;

        let mut label = label_query.get_mut(label_entity).unwrap();
        label.sections[0].value = text.0.clone();
    }
}
