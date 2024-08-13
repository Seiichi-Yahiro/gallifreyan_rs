use crate::plugins::text_converter::SetText;
use crate::plugins::ui::icons::Icons;
use crate::plugins::ui::widgets;
use crate::plugins::ui::{styles, UiRoot};
use bevy::prelude::*;

pub struct SidebarPlugin;

impl Plugin for SidebarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup.in_set(UiSidebarSet));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct UiSidebarSet;

#[derive(Component)]
pub struct Sidebar;

fn setup(mut commands: Commands, ui_root_query: Query<Entity, With<UiRoot>>, icons: Res<Icons>) {
    let root = ui_root_query.get_single().unwrap();

    let left = commands
        .spawn((
            Name::new("Ui Sidebar"),
            Sidebar,
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
        .id();

    let text_input = widgets::text_input::create(&mut commands, "Sentence".to_string());
    commands
        .entity(text_input)
        .set_parent(left)
        .observe(on_sentence_change);

    let tree = commands
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
        .set_parent(left)
        .id();

    let (scroll_area, tree_content) = widgets::scroll_area::create(&mut commands);
    commands.entity(scroll_area).set_parent(tree);

    // TODO remove
    let (foldable, content) =
        widgets::foldable::create(&mut commands, &icons, "Foldable".to_string(), true);
    commands.entity(foldable).set_parent(tree_content);

    for i in 0..20 {
        commands
            .spawn(TextBundle::from_section(
                format!("Row {}", i),
                styles::TEXT_STYLE,
            ))
            .set_parent(content);
    }
}

fn on_sentence_change(
    trigger: Trigger<widgets::text_input::Changed>,
    mut set_text_events: EventWriter<SetText>,
) {
    let text = trigger.event().0.clone();
    set_text_events.send(SetText(text));
}
