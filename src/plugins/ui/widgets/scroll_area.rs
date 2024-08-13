use crate::plugins::ui::interactions::{HoverIn, HoverOut};
use crate::plugins::ui::styles;
use bevy::ecs::component::StorageType;
use bevy::ecs::query::QuerySingleError;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

const MIN_HANDLE_HEIGHT: f32 = 20.0;
const BAR_WIDTH: f32 = 7.0;

const BAR_COLOR: Color = Color::srgba(
    (2.0 * 42.0) / 255.0,
    (2.0 * 42.0) / 255.0,
    (2.0 * 42.0) / 255.0,
    255.0 / 3.0,
);

const HANDLE_COLOR: Color = Color::srgba(
    (4.0 * 42.0) / 255.0,
    (4.0 * 42.0) / 255.0,
    (4.0 * 42.0) / 255.0,
    255.0 / 3.0,
);

pub struct ScrollAreaPlugin;

impl Plugin for ScrollAreaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (scroll.run_if(on_event::<MouseWheel>()), size_handle),
        );
    }
}

struct Scrollable;

impl Component for Scrollable {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}

#[derive(Component)]
struct ScrollArea {
    content: Entity,
    bar: Entity,
    handle: Entity,
}

#[derive(Component)]
struct ScrollContent;

#[derive(Component)]
struct ScrollBar;

#[derive(Component)]
struct ScrollHandle;

fn activate_scroll(trigger: Trigger<HoverIn>, mut commands: Commands) {
    commands.entity(trigger.entity()).insert(Scrollable);
}

fn deactivate_scroll(trigger: Trigger<HoverOut>, mut commands: Commands) {
    commands.entity(trigger.entity()).remove::<Scrollable>();
}

fn scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    scroll_area_query: Query<(&ScrollArea, &Node), With<Scrollable>>,
    mut style_node_query: Query<(&mut Style, &Node)>,
) {
    match scroll_area_query.get_single() {
        Ok((scroll_area, scroll_area_node)) => {
            let [(mut content_style, content_node), (mut handle_style, handle_node)] =
                style_node_query
                    .get_many_mut([scroll_area.content, scroll_area.handle])
                    .unwrap();

            let current = match content_style.top {
                Val::Px(top) => top,
                _ => {
                    panic!("Scroll offset should be in px");
                }
            };

            let offset = mouse_wheel_events
                .read()
                .map(|event| match event.unit {
                    MouseScrollUnit::Line => event.y * styles::FONT_SIZE,
                    MouseScrollUnit::Pixel => event.y,
                })
                .sum::<f32>();

            let content_min_top = (scroll_area_node.size().y - content_node.size().y).min(0.0);
            let content_top = (current + offset).clamp(content_min_top, 0.0);

            let handle_max_top = scroll_area_node.size().y - handle_node.size().y;
            let handle_top = (content_top / content_min_top) * handle_max_top;

            content_style.top = Val::Px(content_top);
            handle_style.top = Val::Px(handle_top);
        }
        Err(QuerySingleError::MultipleEntities(_)) => {
            error!("Scrolling multiple entities not supported");
        }
        Err(QuerySingleError::NoEntities(_)) => {}
    }
}

// TODO make lazy
fn size_handle(
    scroll_area_query: Query<(&ScrollArea, &Node)>,
    content_query: Query<&Node, With<ScrollContent>>,
    mut bar_query: Query<&mut Visibility, With<ScrollBar>>,
    mut handle_query: Query<&mut Style, With<ScrollHandle>>,
) {
    for (scroll_area, scroll_area_node) in scroll_area_query.iter() {
        let content_node = content_query.get(scroll_area.content).unwrap();
        let mut handle_style = handle_query.get_mut(scroll_area.handle).unwrap();

        let factor = scroll_area_node.size().y / content_node.size().y;
        let size = (scroll_area_node.size().y * factor).max(MIN_HANDLE_HEIGHT);

        handle_style.height = Val::Px(size);

        let mut bar_visibility = bar_query.get_mut(scroll_area.bar).unwrap();

        if factor == 1.0 {
            *bar_visibility = Visibility::Hidden;
        } else {
            *bar_visibility = Visibility::Visible;
        }
    }
}

pub fn create(commands: &mut Commands) -> (Entity, Entity) {
    let content = commands
        .spawn((
            Name::new("Scroll Area Content"),
            ScrollContent,
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_self: AlignSelf::Stretch,
                    top: Val::Px(0.0),
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    let scroll_handle = commands
        .spawn((
            Name::new("Scroll Handle"),
            ScrollHandle,
            NodeBundle {
                style: Style {
                    display: Display::Block,
                    width: Val::Px(BAR_WIDTH),
                    height: Val::Px(MIN_HANDLE_HEIGHT),
                    top: Val::Px(0.0),
                    ..default()
                },
                background_color: BackgroundColor(HANDLE_COLOR),
                focus_policy: FocusPolicy::Block,
                ..default()
            },
        ))
        .id();

    let scroll_bar = commands
        .spawn((
            Name::new("Scroll Bar"),
            ScrollBar,
            NodeBundle {
                style: Style {
                    display: Display::Block,
                    position_type: PositionType::Absolute,
                    width: Val::Px(BAR_WIDTH),
                    height: Val::Percent(100.0),
                    right: Val::Px(0.0),
                    ..default()
                },
                background_color: BackgroundColor(BAR_COLOR),
                focus_policy: FocusPolicy::Block,
                ..default()
            },
        ))
        .add_child(scroll_handle)
        .id();

    let scroll_area = commands
        .spawn((
            Name::new("Scroll Area Container"),
            ScrollArea {
                content,
                bar: scroll_bar,
                handle: scroll_handle,
            },
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_self: AlignSelf::Stretch,
                    overflow: Overflow {
                        x: OverflowAxis::Hidden,
                        y: OverflowAxis::Hidden,
                    },
                    ..default()
                },
                focus_policy: FocusPolicy::Block,
                ..default()
            },
            Interaction::None,
        ))
        .observe(activate_scroll)
        .observe(deactivate_scroll)
        .push_children(&[content, scroll_bar])
        .id();

    (scroll_area, content)
}
