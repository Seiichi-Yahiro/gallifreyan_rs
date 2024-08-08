use std::time::Duration;

use crate::plugins::ui::interactions::Pressed;
use crate::plugins::ui::styles;
use ab_glyph::{Font as AbFont, ScaleFont};
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy::text::BreakLineOn;
use bevy::ui::RelativeCursorPosition;
use unicode_segmentation::UnicodeSegmentation;

pub struct TextInputPlugin;

impl Plugin for TextInputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FocusedTextInput(None))
            .add_event::<FocusEvent>()
            .observe(create_widget)
            .add_systems(
                Update,
                (
                    handle_keyboard_input.run_if(on_event::<KeyboardInput>()),
                    blink_cursor,
                )
                    .chain(),
            )
            .add_systems(PostUpdate, unfocus.run_if(on_event::<MouseButtonInput>()));

        #[cfg(debug_assertions)]
        {
            use crate::debug::debug_log_observer;

            app.observe(debug_log_observer::<Focus>)
                .observe(debug_log_observer::<Blur>)
                .observe(debug_log_observer::<Changed>);
        }
    }
}

#[derive(Component)]
pub struct TextInputWidget {
    name: Option<String>,
}

impl TextInputWidget {
    pub fn new(name: Option<String>) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref().map(String::as_ref).unwrap_or("Unnamed")
    }
}

fn create_widget(
    trigger: Trigger<OnAdd, TextInputWidget>,
    mut commands: Commands,
    widget_query: Query<&TextInputWidget>,
) {
    const CURSOR_WIDTH: f32 = 1.0;

    let widget = widget_query.get(trigger.entity()).unwrap();

    let text = commands
        .spawn((
            Name::new(format!("Text Input Text: {}", widget.name())),
            TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: styles::FONT_HANDLE,
                            font_size: styles::FONT_SIZE,
                            color: styles::FONT_COLOR,
                        },
                    }],
                    justify: JustifyText::Left,
                    linebreak_behavior: BreakLineOn::NoWrap,
                },
                style: Style {
                    left: Val::Px(0.0),
                    ..default()
                },
                ..default()
            },
            TextInputText,
            RelativeCursorPosition::default(),
        ))
        .id();

    let cursor = commands
        .spawn((
            Name::new(format!("Text Input Cursor: {}", widget.name())),
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Px(CURSOR_WIDTH),
                    height: Val::Px(styles::FONT_SIZE),
                    align_self: AlignSelf::Center,
                    left: Val::Px(0.0),
                    ..default()
                },
                background_color: BackgroundColor(styles::FONT_COLOR),
                visibility: Visibility::Hidden,
                ..default()
            },
            CursorPos { glyph_index: 0 },
            CursorTimer {
                timer: Timer::new(Duration::from_millis(500), TimerMode::Repeating),
                reset: false,
            },
        ))
        .id();

    let inner_node = commands
        .spawn((
            Name::new(format!("Text Input Inner Node: {}", widget.name())),
            NodeBundle {
                style: Style {
                    overflow: Overflow::clip(),
                    width: Val::Percent(100.0),
                    height: Val::Px(styles::FONT_SIZE),
                    ..default()
                },
                ..default()
            },
        ))
        .push_children(&[text, cursor])
        .id();

    commands
        .entity(trigger.entity())
        .insert((
            Name::new(format!("Text Input: {}", widget.name())),
            NodeBundle {
                style: Style {
                    min_width: Val::Px(50.0),
                    width: Val::Percent(100.0),
                    height: Val::Px(styles::FONT_SIZE + styles::PADDING * 2.0),
                    border: UiRect::all(Val::Px(styles::BORDER_SIZE)),
                    padding: UiRect::all(Val::Px(styles::PADDING)),
                    ..default()
                },
                background_color: BackgroundColor(styles::BACKGROUND_COLOR),
                border_color: BorderColor(styles::BORDER_COLOR),
                border_radius: BorderRadius::all(Val::Px(styles::BORDER_RADIUS)),
                ..default()
            },
            TextInput {
                inner_node,
                text_entity: text,
                cursor_entity: cursor,
            },
            Interaction::None,
        ))
        .add_child(inner_node)
        .observe(focus)
        .observe(move_cursor_on_click)
        .observe(handle_text_input_events)
        .observe(set_cursor_position)
        .observe(show_cursor_on_focus)
        .observe(hide_cursor_on_blur);
}

#[derive(Resource)]
struct FocusedTextInput(Option<Entity>);

#[derive(Component)]
struct TextInput {
    inner_node: Entity,
    text_entity: Entity,
    cursor_entity: Entity,
}

#[derive(Component)]
struct TextInputText;

#[derive(Debug, Event)]
pub struct Changed(pub String);

#[derive(Debug, Event)]
enum TextInputEvent {
    Text(TextAction),
    Cursor(CursorMovement),
}

#[derive(Debug)]
enum TextAction {
    Add(String),
    DeleteGlyphs(isize),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum CursorMovement {
    Start,
    End,
    Relative {
        offset: isize,
        unit: CursorRelativeMovementUnit,
    },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum CursorRelativeMovementUnit {
    Glyph,
    Word,
}

#[derive(Component)]
struct CursorPos {
    glyph_index: usize,
}

#[derive(Component)]
struct CursorTimer {
    timer: Timer,
    reset: bool,
}

#[derive(Debug, Event)]
struct FocusEvent;

#[derive(Debug, Event)]
struct Focus;

#[derive(Debug, Event)]
struct Blur;

fn focus(
    trigger: Trigger<Pressed>,
    mut commands: Commands,
    mut focused_text_input: ResMut<FocusedTextInput>,
    mut focus_events: EventWriter<FocusEvent>,
) {
    let entity = trigger.entity();

    if focused_text_input
        .0
        .is_some_and(|prev_entity| prev_entity == entity)
    {
        focus_events.send(FocusEvent);
        return;
    }

    focused_text_input.0 = Some(entity);
    focus_events.send(FocusEvent);
    commands.trigger_targets(Focus, entity);
}

fn unfocus(
    mut commands: Commands,
    mut focused_text_input: ResMut<FocusedTextInput>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut focus_events: EventReader<FocusEvent>,
) {
    if focused_text_input.0.is_none() {
        return;
    }

    for mouse_event in mouse_button_events.read() {
        if mouse_event.button != MouseButton::Left || !mouse_event.state.is_pressed() {
            continue;
        }

        if !focus_events.is_empty() {
            focus_events.clear();
            return;
        }

        let entity = focused_text_input.0.take().unwrap();
        commands.trigger_targets(Blur, entity);
    }
}

fn handle_keyboard_input(
    mut commands: Commands,
    focused_text_input: Res<FocusedTextInput>,
    mut keyboard_events: EventReader<KeyboardInput>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let Some(focused_entity) = focused_text_input.0 else {
        return;
    };

    for event in keyboard_events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        let ctrl = keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

        match event.key_code {
            KeyCode::ArrowLeft => {
                commands.trigger_targets(
                    TextInputEvent::Cursor(CursorMovement::Relative {
                        offset: -1,
                        unit: if ctrl {
                            CursorRelativeMovementUnit::Word
                        } else {
                            CursorRelativeMovementUnit::Glyph
                        },
                    }),
                    focused_entity,
                );
            }
            KeyCode::ArrowRight => {
                commands.trigger_targets(
                    TextInputEvent::Cursor(CursorMovement::Relative {
                        offset: 1,
                        unit: if ctrl {
                            CursorRelativeMovementUnit::Word
                        } else {
                            CursorRelativeMovementUnit::Glyph
                        },
                    }),
                    focused_entity,
                );
            }
            KeyCode::Home => {
                commands.trigger_targets(
                    TextInputEvent::Cursor(CursorMovement::Start),
                    focused_entity,
                );
            }
            KeyCode::End => {
                commands
                    .trigger_targets(TextInputEvent::Cursor(CursorMovement::End), focused_entity);
            }
            KeyCode::Backspace => {
                commands.trigger_targets(
                    TextInputEvent::Text(TextAction::DeleteGlyphs(-1)),
                    focused_entity,
                );
            }
            KeyCode::Delete => {
                commands.trigger_targets(
                    TextInputEvent::Text(TextAction::DeleteGlyphs(1)),
                    focused_entity,
                );
            }
            KeyCode::Space => {
                commands.trigger_targets(
                    TextInputEvent::Text(TextAction::Add(" ".to_string())),
                    focused_entity,
                );
            }
            _ => {
                if let Key::Character(s) = &event.logical_key {
                    commands.trigger_targets(
                        TextInputEvent::Text(TextAction::Add(s.to_string())),
                        focused_entity,
                    );
                }
            }
        }
    }
}

fn handle_text_input_events(
    trigger: Trigger<TextInputEvent>,
    mut commands: Commands,
    text_input_query: Query<&TextInput>,
    mut text_query: Query<&mut Text>,
    mut cursor_query: Query<(&mut CursorPos, &mut CursorTimer)>,
) {
    let text_input = text_input_query.get(trigger.entity()).unwrap();
    let mut text = text_query.get_mut(text_input.text_entity).unwrap();
    let (mut cursor_pos, mut cursor_timer) =
        cursor_query.get_mut(text_input.cursor_entity).unwrap();

    let get_byte_index = |glyph_index: usize| -> usize {
        text.sections[0]
            .value
            .grapheme_indices(true)
            .skip(glyph_index)
            .map(|it| it.0)
            .next()
            .unwrap_or(text.sections[0].value.len())
    };

    match trigger.event() {
        TextInputEvent::Text(TextAction::Add(text_addition)) => {
            let byte_index = get_byte_index(cursor_pos.glyph_index);
            text.sections[0].value.insert_str(byte_index, text_addition);
            let graphemes = text_addition.graphemes(true).count();
            cursor_pos.glyph_index += graphemes;

            commands.trigger_targets(Changed(text.sections[0].value.clone()), trigger.entity());
        }
        TextInputEvent::Text(TextAction::DeleteGlyphs(offset)) => {
            let current = get_byte_index(cursor_pos.glyph_index);
            let after = get_byte_index(cursor_pos.glyph_index.saturating_add_signed(*offset));

            if *offset < 0 {
                text.sections[0].value.drain(after..current);
                cursor_pos.glyph_index = cursor_pos.glyph_index.saturating_add_signed(*offset);

                commands.trigger_targets(Changed(text.sections[0].value.clone()), trigger.entity());
            } else if *offset > 0 {
                text.sections[0].value.drain(current..after);

                commands.trigger_targets(Changed(text.sections[0].value.clone()), trigger.entity());
            }
        }
        TextInputEvent::Cursor(CursorMovement::Start) => {
            cursor_pos.glyph_index = 0;
        }
        TextInputEvent::Cursor(CursorMovement::End) => {
            cursor_pos.glyph_index = text.sections[0].value.graphemes(true).count();
        }
        TextInputEvent::Cursor(CursorMovement::Relative { offset, unit }) => match unit {
            CursorRelativeMovementUnit::Glyph => {
                cursor_pos.glyph_index = cursor_pos
                    .glyph_index
                    .saturating_add_signed(*offset)
                    .min(text.sections[0].value.graphemes(true).count());
            }
            CursorRelativeMovementUnit::Word => {
                let byte_index = get_byte_index(cursor_pos.glyph_index);

                let glyph_offset: isize = if *offset < 0 {
                    -text.sections[0].value[..byte_index]
                        .split_word_bounds()
                        .rev()
                        .map(|word| word.graphemes(true).count() as isize)
                        .take((-offset) as usize)
                        .sum::<isize>()
                } else {
                    text.sections[0].value[byte_index..]
                        .split_word_bounds()
                        .map(|word| word.graphemes(true).count() as isize)
                        .take(*offset as usize)
                        .sum::<isize>()
                };

                cursor_pos.glyph_index = cursor_pos.glyph_index.saturating_add_signed(glyph_offset);
            }
        },
    }

    cursor_timer.reset = true;
    commands.trigger_targets(PositionCursor, trigger.entity());
}

#[derive(Event)]
struct PositionCursor;

fn set_cursor_position(
    trigger: Trigger<PositionCursor>,
    text_input_query: Query<&TextInput>,
    mut text_query: Query<(&Text, &mut Style), Without<CursorPos>>,
    mut cursor_query: Query<(&CursorPos, &Node, &mut Style), Without<Text>>,
    inner_node_query: Query<&Node>,
    fonts: Res<Assets<Font>>,
) {
    let text_input = text_input_query.get(trigger.entity()).unwrap();

    let (text, mut text_style) = text_query.get_mut(text_input.text_entity).unwrap();
    let text_section = &text.sections[0];

    let font = fonts.get(&text_section.style.font).unwrap();
    let scaled_font = font.font.clone().into_scaled(text_section.style.font_size);

    let (cursor_pos, cursor_node, mut cursor_style) =
        cursor_query.get_mut(text_input.cursor_entity).unwrap();

    let cursor_width = cursor_node.size().x;

    let inner_node = inner_node_query.get(text_input.inner_node).unwrap();
    let text_box_width = inner_node.size().x - cursor_width;

    let text_left = match text_style.left {
        Val::Px(left) => left,
        _ => {
            panic!("text style left should be in px");
        }
    };

    let pos_in_text = text_section
        .value
        .graphemes(true)
        .take(cursor_pos.glyph_index)
        .map(|ch| scaled_font.glyph_id(ch.chars().next().unwrap()))
        .map(|glyph| scaled_font.h_advance(glyph))
        .sum::<f32>();

    let cursor_left = pos_in_text + text_left - cursor_width;

    let text_offset = if cursor_left < cursor_width {
        text_left - cursor_left
    } else if cursor_left > text_box_width {
        text_left - (cursor_left - text_box_width)
    } else {
        text_left
    };

    cursor_style.left = Val::Px(cursor_left.clamp(0.0, text_box_width));
    text_style.left = Val::Px(text_offset);
}

fn show_cursor_on_focus(
    trigger: Trigger<Focus>,
    mut text_input_query: Query<&TextInput>,
    mut cursor_query: Query<&mut Visibility>,
) {
    let text_input = text_input_query.get_mut(trigger.entity()).unwrap();
    let mut cursor_visibility = cursor_query.get_mut(text_input.cursor_entity).unwrap();
    *cursor_visibility = Visibility::Visible;
}

fn hide_cursor_on_blur(
    trigger: Trigger<Blur>,
    mut text_input_query: Query<&TextInput>,
    mut cursor_query: Query<&mut Visibility>,
) {
    let text_input = text_input_query.get_mut(trigger.entity()).unwrap();
    let mut cursor_visibility = cursor_query.get_mut(text_input.cursor_entity).unwrap();
    *cursor_visibility = Visibility::Hidden;
}

fn blink_cursor(
    focused_text_input: Res<FocusedTextInput>,
    text_input_query: Query<&TextInput>,
    mut cursor_query: Query<(&mut Visibility, &mut CursorTimer)>,
    time: Res<Time>,
) {
    let Some(focused_entity) = focused_text_input.0 else {
        return;
    };

    let text_input = text_input_query.get(focused_entity).unwrap();
    let (mut cursor_visibility, mut cursor_timer) =
        cursor_query.get_mut(text_input.cursor_entity).unwrap();

    if cursor_timer.reset {
        cursor_timer.reset = false;
        cursor_timer.timer.reset();
        *cursor_visibility = Visibility::Visible;
    } else if cursor_timer.timer.tick(time.delta()).just_finished() {
        *cursor_visibility = if *cursor_visibility == Visibility::Hidden {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn move_cursor_on_click(
    trigger: Trigger<Pressed>,
    text_input_query: Query<&TextInput>,
    text_query: Query<(&Text, &Node, &RelativeCursorPosition)>,
    mut cursor_query: Query<(&mut CursorPos, &mut CursorTimer)>,
    fonts: Res<Assets<Font>>,
    mut commands: Commands,
) {
    let text_input = text_input_query.get(trigger.entity()).unwrap();
    let (text, text_node, relative_cursor_position) =
        text_query.get(text_input.text_entity).unwrap();

    let Some(relative_pos) = relative_cursor_position.normalized else {
        return;
    };

    if relative_pos.x < 0.0 || relative_pos.x > 1.0 {
        return;
    }

    let cursor_pos_in_text = text_node.size().x * relative_pos.x;

    let text_section = &text.sections[0];
    let font = fonts.get(&text_section.style.font).unwrap();
    let scaled_font = font.font.clone().into_scaled(text_section.style.font_size);

    let (mut cursor_pos, mut cursor_timer) =
        cursor_query.get_mut(text_input.cursor_entity).unwrap();

    cursor_pos.glyph_index = text_section
        .value
        .graphemes(true)
        .map(|ch| scaled_font.glyph_id(ch.chars().next().unwrap()))
        .map(|glyph| scaled_font.h_advance(glyph))
        .scan((0.0f32, 0.0f32), |state, width| {
            state.0 = state.1;
            state.1 += width;
            Some(*state)
        })
        .enumerate()
        .find_map(|(i, (left, right))| {
            if cursor_pos_in_text >= left && cursor_pos_in_text <= right {
                let diff_left = cursor_pos_in_text - left;
                let diff_right = right - cursor_pos_in_text;

                if diff_left <= diff_right {
                    Some(i)
                } else {
                    Some(i + 1)
                }
            } else {
                None
            }
        })
        .unwrap_or(0);

    cursor_timer.reset = true;
    commands.trigger_targets(PositionCursor, trigger.entity());
}
