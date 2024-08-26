mod tree;

use crate::plugins::text_converter::prelude::SetSentence;
use crate::plugins::ui::sidebar::tree::TreePlugin;
use crate::plugins::ui::widgets;
use crate::plugins::ui::{styles, UiRoot};
use bevy::prelude::*;

pub struct SidebarPlugin;

impl Plugin for SidebarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup.in_set(UiSidebarSet))
            .add_plugins(TreePlugin);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct UiSidebarSet;

#[derive(Component)]
pub struct Sidebar;

fn setup(mut commands: Commands, ui_root_query: Query<Entity, With<UiRoot>>) {
    let root = ui_root_query.get_single().unwrap();

    let text_input = widgets::text_input::create(&mut commands, "Sentence".to_string());
    commands.entity(text_input).observe(on_sentence_change);

    commands
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
        .add_child(text_input);
}

fn on_sentence_change(
    trigger: Trigger<widgets::text_input::Changed>,
    mut set_text_events: EventWriter<SetSentence>,
) {
    let text = trigger.event().0.clone();
    set_text_events.send(SetSentence(text));
}
