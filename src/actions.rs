mod reset;

use crate::actions::reset::*;
use crate::event_set::*;
use crate::text_converter;
use crate::ui::{UiSize, UiState};
use bevy::prelude::*;

use std::marker::PhantomData;

pub struct ActionsPlugin<T: Component> {
    phantom: PhantomData<T>,
}

impl<T: Component> Default for ActionsPlugin<T> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<T: Component> Plugin for ActionsPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_event_set::<Actions>()
            .add_system(Self::adjust_root_to_ui)
            .add_system_to_stage(CoreStage::PreUpdate, Self::set_text_action)
            .add_plugin(ResetPlugin);
    }
}

pub struct SetText(pub String);
pub struct UiSizeChanged(pub UiSize);

event_set!(Actions {
    SetText,
    UiSizeChanged,
    ResetAll
});

impl<T: Component> ActionsPlugin<T> {
    fn adjust_root_to_ui(
        mut root_query: Query<&mut Transform, With<T>>,
        mut ui_size_event: EventReader<UiSizeChanged>,
    ) {
        if let Some(UiSizeChanged(ui_size)) = ui_size_event.iter().last() {
            let mut root = root_query.single_mut();
            root.translation.x = ui_size.sidebar_width / 2.0;
        }
    }

    fn set_text_action(
        mut commands: Commands,
        mut events: EventReader<SetText>,
        mut actions: EventWriter<ResetAll>,
        mut ui_state: ResMut<UiState>,
        root_query: Query<Entity, With<T>>,
    ) {
        if let Some(SetText(text)) = events.iter().last() {
            let root = root_query.single();
            commands.entity(root).despawn_descendants();

            if !text.is_empty() {
                let sentence_node = text_converter::spawn_sentence(&mut commands, text);
                commands.entity(root).add_child(sentence_node.entity);
                ui_state.tree = Some(sentence_node);

                actions.send(ResetAll);
            } else {
                ui_state.tree = None;
            }
        }
    }
}
