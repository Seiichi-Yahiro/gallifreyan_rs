use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_egui::egui::Color32;
use bevy_egui::{egui, EguiContext};

pub struct ColorThemePlugin;

impl Plugin for ColorThemePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::rgb_u8(27, 27, 27)))
            .init_resource::<ColorTheme>()
            .add_startup_system(setup_color_theme)
            .add_system(update_clear_color);
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Theme {
    Dark,
    Light,
}

pub const BACKGROUND_COLOR: &str = "BACKGROUND_COLOR";
pub const DRAW_COLOR: &str = "DRAW_COLOR";
pub const SELECT_COLOR: &str = "SELECT_COLOR";

#[derive(Component)]
pub struct ColorDependency(pub &'static str);

#[derive(Debug, Resource)]
pub struct ColorTheme {
    theme: Theme,
    values: HashMap<&'static str, (Color, Color)>,
}

impl ColorTheme {
    pub fn current(&self) -> Theme {
        self.theme
    }

    pub fn set_theme(&mut self, theme: Theme, ctx: &egui::Context) {
        let visuals = match theme {
            Theme::Dark => egui::Visuals::dark(),
            Theme::Light => egui::Visuals::light(),
        };

        ctx.set_visuals(visuals);

        self.theme = theme;
    }

    pub fn insert(&mut self, key: &'static str, dark: Color, light: Color) {
        self.values.insert(key, (dark, light));
    }

    pub fn get(&self, key: &str) -> Option<Color> {
        self.values.get(key).map(|(dark, light)| match self.theme {
            Theme::Dark => *dark,
            Theme::Light => *light,
        })
    }
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            values: HashMap::new(),
        }
    }
}

fn setup_color_theme(mut color_theme: ResMut<ColorTheme>, mut egui_context: ResMut<EguiContext>) {
    color_theme.set_theme(Theme::Dark, egui_context.ctx_mut());

    let dark = egui::Visuals::dark();
    let light = egui::Visuals::light();

    color_theme.insert(
        BACKGROUND_COLOR,
        dark.widgets.noninteractive.bg_fill.into_color(),
        light.widgets.noninteractive.bg_fill.into_color(),
    );

    color_theme.insert(DRAW_COLOR, Color::WHITE, Color::BLACK);

    color_theme.insert(
        SELECT_COLOR,
        dark.selection.bg_fill.into_color(),
        light.selection.bg_fill.into_color(),
    );
}

fn update_clear_color(mut clear_color: ResMut<ClearColor>, color_theme: Res<ColorTheme>) {
    if !color_theme.is_changed() {
        return;
    }

    *clear_color = ClearColor(color_theme.get(BACKGROUND_COLOR).unwrap());
}

pub trait IntoColor {
    fn into_color(self) -> Color;
}

impl IntoColor for Color32 {
    fn into_color(self) -> Color {
        Color::rgba_u8(self.r(), self.g(), self.b(), self.a())
    }
}
