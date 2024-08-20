use crate::plugins::ui::icons::Icons;
use crate::plugins::ui::interactions::{HoverIn, HoverOut, Pressed};
use crate::plugins::ui::styles;
use bevy::prelude::*;

#[derive(Component)]
pub struct Foldable {
    pub header: Entity,
    pub label: Entity,
    pub content: Entity,
}

#[derive(Component)]
pub struct FoldableHeader;

#[derive(Component)]
pub struct FoldableLabel;

#[derive(Component)]
pub struct FoldableContent;

#[derive(Component)]
struct Opened(bool);

#[derive(Component)]
struct Link(Entity);

pub fn create(
    commands: &mut Commands,
    icons: &Res<Icons>,
    label: String,
    is_open: bool,
) -> (Entity, Entity) {
    let level_line = commands
        .spawn((
            Name::new("Foldable Level Line"),
            NodeBundle {
                style: Style {
                    width: Val::Px(1.0),
                    height: Val::Percent(100.0),
                    align_self: AlignSelf::Center,
                    ..default()
                },
                background_color: BackgroundColor(styles::FONT_COLOR),
                ..default()
            },
        ))
        .id();

    let level_line_container = commands
        .spawn((
            Name::new("Foldable Level Line Container"),
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    width: Val::Px(styles::FONT_SIZE),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            },
        ))
        .add_child(level_line)
        .id();

    let content = commands
        .spawn((
            Name::new("Foldable Content"),
            FoldableContent,
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    let content_container = commands
        .spawn((
            Name::new("Foldable Content Container"),
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(styles::PADDING),
                    width: Val::Percent(100.0),
                    height: if is_open { Val::Auto } else { Val::ZERO },
                    ..default()
                },
                ..default()
            },
        ))
        .push_children(&[level_line_container, content])
        .id();

    let chevron = commands
        .spawn((
            Name::new("Foldable Chevron"),
            ImageBundle {
                style: Style {
                    width: Val::Px(styles::FONT_SIZE),
                    height: Val::Px(styles::FONT_SIZE),
                    ..default()
                },
                image: UiImage {
                    color: styles::FONT_COLOR,
                    texture: if is_open {
                        icons.chevron_down.clone()
                    } else {
                        icons.chevron_right.clone()
                    },
                    ..default()
                },
                ..default()
            },
            Opened(is_open),
            Link(content_container),
            Interaction::None,
        ))
        .observe(open_close)
        .observe(highlight_chevron_on_hover_in)
        .observe(remove_highlight_chevron_on_hover_out)
        .id();

    let header_label = commands
        .spawn((
            Name::new("Foldable label"),
            FoldableLabel,
            TextBundle::from_section(label, styles::TEXT_STYLE),
        ))
        .id();

    let header = commands
        .spawn((
            Name::new("Foldable Header"),
            FoldableHeader,
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(styles::PADDING),
                    width: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            },
        ))
        .push_children(&[chevron, header_label])
        .id();

    let foldable = commands
        .spawn((
            Name::new("Foldable"),
            Foldable {
                header,
                label: header_label,
                content,
            },
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            },
        ))
        .push_children(&[header, content_container])
        .id();

    (foldable, content)
}

fn open_close(
    trigger: Trigger<Pressed>,
    mut chevron_query: Query<(&mut UiImage, &mut Opened, &Link)>,
    mut content_container_query: Query<&mut Style>,
    icons: Res<Icons>,
) {
    let (mut image, mut opened, content_container_link) =
        chevron_query.get_mut(trigger.entity()).unwrap();

    opened.0 = !opened.0;

    let mut style = content_container_query
        .get_mut(content_container_link.0)
        .unwrap();

    if opened.0 {
        image.texture = icons.chevron_down.clone();
        style.height = Val::Auto;
    } else {
        image.texture = icons.chevron_right.clone();
        style.height = Val::ZERO;
    };
}

fn highlight_chevron_on_hover_in(
    trigger: Trigger<HoverIn>,
    mut chevron_query: Query<&mut UiImage>,
) {
    let mut image = chevron_query.get_mut(trigger.entity()).unwrap();
    image.color = styles::HIGHLIGHT_COLOR;
}

fn remove_highlight_chevron_on_hover_out(
    trigger: Trigger<HoverOut>,
    mut chevron_query: Query<&mut UiImage>,
) {
    let mut image = chevron_query.get_mut(trigger.entity()).unwrap();
    image.color = styles::FONT_COLOR;
}

/*#[derive(Debug)]
pub enum FoldableContent {
    Add { index: usize, child: Entity },
    Remove(Entity),
}

impl EntityCommand for FoldableContent {
    fn apply(self, id: Entity, world: &mut World) {
        let content = world.entity(id).get::<Foldable>().unwrap().content;

        let content_count = world
            .entity(content)
            .get::<Children>()
            .map(|children| children.len())
            .unwrap_or(0);

        let mut content = world.commands().entity(content);

        match self {
            FoldableContent::Add { index, child } => {
                if index >= content_count {
                    content.add_child(child);
                } else {
                    content.insert_children(index, &[child]);
                }
            }
            FoldableContent::Remove(child) => {
                content.remove_children(&[child]);
            }
        }

        world.flush_commands();
    }
}*/
