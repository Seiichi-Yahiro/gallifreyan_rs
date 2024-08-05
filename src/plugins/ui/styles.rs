use bevy::asset::Handle;
use bevy::color::Color;
use bevy::prelude::Font;

pub const FONT_HANDLE: Handle<Font> = Handle::weak_from_u128(10482756907980398621);
pub const FONT_SIZE: f32 = 16.0;
pub const FONT_COLOR: Color = Color::WHITE;
pub const BACKGROUND_COLOR: Color = Color::srgb(42.0 / 255.0, 42.0 / 255.0, 42.0 / 255.0);
pub const PADDING: f32 = 4.0;
pub const BORDER_RADIUS: f32 = 4.0;
pub const BORDER_SIZE: f32 = 1.0;
pub const BORDER_COLOR: Color = Color::BLACK;
