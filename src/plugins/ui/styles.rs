use bevy::asset::Handle;
use bevy::color::Color;
use bevy::prelude::{Font, TextStyle};

pub const FONT_HANDLE: Handle<Font> = Handle::weak_from_u128(10482756907980398621);
pub const FONT_SIZE: f32 = 16.0;
pub const FONT_COLOR: Color = Color::WHITE;

pub const TEXT_STYLE: TextStyle = TextStyle {
    font: FONT_HANDLE,
    font_size: FONT_SIZE,
    color: FONT_COLOR,
};

pub const BACKGROUND_COLOR: Color = Color::srgb(42.0 / 255.0, 42.0 / 255.0, 42.0 / 255.0);
pub const PADDING: f32 = 4.0;
pub const BORDER_RADIUS: f32 = 4.0;
pub const BORDER_SIZE: f32 = 1.0;
pub const BORDER_COLOR: Color = Color::BLACK;

pub const HIGHLIGHT_COLOR: Color = Color::srgb(0.2, 0.6, 1.0);
