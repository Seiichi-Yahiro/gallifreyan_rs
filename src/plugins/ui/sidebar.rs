use crate::plugins::text_converter::SetText;
use crate::plugins::ui::text_input::TextInputWidget;
use crate::plugins::ui::{styles, text_input, UiRoot};
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

fn setup(mut commands: Commands, ui_root_query: Query<Entity, With<UiRoot>>) {
    let root = ui_root_query.get_single().unwrap();

    let left = commands
        .spawn((
            Name::new("Ui Sidebar"),
            Sidebar,
            NodeBundle {
                style: Style {
                    width: Val::Percent(20.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(styles::PADDING)),
                    border: UiRect::right(Val::Px(styles::BORDER_SIZE)),
                    ..default()
                },
                background_color: BackgroundColor(styles::BACKGROUND_COLOR),
                border_color: BorderColor(styles::BORDER_COLOR),
                ..default()
            },
        ))
        .set_parent(root)
        .id();

    commands
        .spawn(TextInputWidget::new(Some("Sentence".to_string())))
        .set_parent(left)
        .observe(on_sentence_change);
}

fn on_sentence_change(trigger: Trigger<text_input::Changed>, mut commands: Commands) {
    let text = trigger.event().0.clone();
    commands.trigger(SetText(text));
}
