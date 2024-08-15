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
            (
                detect_content_size_changed,
                handle_scroll_events.run_if(on_event::<MouseWheel>()),
                update_view,
                update_handle,
            )
                .chain(),
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
    offset: f32,
    overflow: f32,
    visible_area: f32,
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

fn detect_content_size_changed(
    content_query: Query<&Parent, (Changed<Node>, With<ScrollContent>)>,
    mut scroll_area_query: Query<&mut ScrollArea>,
) {
    for parent in content_query.iter() {
        let mut scroll_area = scroll_area_query.get_mut(parent.get()).unwrap();
        scroll_area.set_changed();
    }
}

fn handle_scroll_events(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut scroll_area_query: Query<&mut ScrollArea, With<Scrollable>>,
) {
    match scroll_area_query.get_single_mut() {
        Ok(mut scroll_area) => {
            let offset = mouse_wheel_events
                .read()
                .map(|event| match event.unit {
                    MouseScrollUnit::Line => event.y * styles::FONT_SIZE,
                    MouseScrollUnit::Pixel => event.y,
                })
                .sum::<f32>();

            scroll_area.offset -= offset;
        }
        Err(QuerySingleError::MultipleEntities(_)) => {
            error!("Scrolling multiple entities not supported");
        }
        Err(QuerySingleError::NoEntities(_)) => {}
    }
}

fn update_view(
    mut scroll_area_query: Query<
        (&mut ScrollArea, &Node),
        Or<(Changed<ScrollArea>, Changed<Node>)>,
    >,
    mut content_query: Query<(&mut Style, &Node), With<ScrollContent>>,
) {
    for (mut scroll_area, scroll_area_node) in scroll_area_query.iter_mut() {
        let (mut content_style, content_node) = content_query.get_mut(scroll_area.content).unwrap();

        scroll_area.overflow = (content_node.size().y - scroll_area_node.size().y).max(0.0);

        scroll_area.offset = scroll_area.offset.clamp(0.0, scroll_area.overflow);

        scroll_area.visible_area =
            (scroll_area_node.size().y / content_node.size().y).clamp(0.0, 1.0);

        content_style.top = Val::Px(-scroll_area.offset);
    }
}

fn update_handle(
    scroll_area_query: Query<&ScrollArea, Or<(Changed<ScrollArea>, Changed<Node>)>>,
    mut bar_query: Query<(&mut Visibility, &Node), With<ScrollBar>>,
    mut handle_query: Query<&mut Style, With<ScrollHandle>>,
) {
    for scroll_area in scroll_area_query.iter() {
        let mut handle_style = handle_query.get_mut(scroll_area.handle).unwrap();

        let (mut bar_visibility, bar_node) = bar_query.get_mut(scroll_area.bar).unwrap();

        let bar_height = bar_node.size().y;
        let handle_height = (bar_height * scroll_area.visible_area).max(MIN_HANDLE_HEIGHT);

        let scroll_offset_percent = if scroll_area.overflow == 0.0 {
            0.0
        } else {
            scroll_area.offset / scroll_area.overflow
        };

        let handle_offset = scroll_offset_percent * (bar_height - handle_height);

        handle_style.height = Val::Px(handle_height);
        handle_style.top = Val::Px(handle_offset);

        if scroll_area.visible_area == 1.0 {
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
                offset: 0.0,
                overflow: 0.0,
                visible_area: 1.0,
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
