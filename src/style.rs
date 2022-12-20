use bevy::prelude::*;
use bevy_egui::egui::Color32;
use bevy_egui::{egui, EguiContext};

pub struct StylePlugin;

impl Plugin for StylePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetTheme>()
            .insert_resource(ClearColor(Color::rgb_u8(27, 27, 27)))
            .init_resource::<Styles>()
            .add_startup_system(setup_styles)
            .add_startup_system(init_clear_color.after(setup_styles))
            .add_system_to_stage(CoreStage::PostUpdate, set_theme);
    }
}

pub const DEFAULT_SVG_COLOR: Color = Color::WHITE;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

pub struct SetTheme(pub Theme);

#[derive(Debug, Default, Resource)]
pub struct Styles {
    pub theme: Theme,
    pub svg_background_color: Color,
    pub svg_color: Color,
    pub selection_color: Color,
}

fn setup_styles(mut styles: ResMut<Styles>, mut egui_context: ResMut<EguiContext>) {
    let ctx = egui_context.ctx_mut();
    ctx.set_visuals(egui::Visuals::dark());

    let style = ctx.style();

    let svg_background_color = color_from(style.visuals.widgets.noninteractive.bg_fill);
    let selection_color = color_from(style.visuals.selection.bg_fill);

    *styles = Styles {
        theme: Theme::Dark,
        svg_background_color,
        svg_color: DEFAULT_SVG_COLOR,
        selection_color,
    }
}

fn set_theme(
    mut events: EventReader<SetTheme>,
    mut styles: ResMut<Styles>,
    mut egui_context: ResMut<EguiContext>,
    mut clear_color: ResMut<ClearColor>,
) {
    if let Some(SetTheme(theme)) = events.iter().last() {
        let visuals = match theme {
            Theme::Dark => egui::Visuals::dark(),
            Theme::Light => egui::Visuals::light(),
        };

        let ctx = egui_context.ctx_mut();
        ctx.set_visuals(visuals);

        let style = ctx.style();

        let svg_background_color = color_from(style.visuals.widgets.noninteractive.bg_fill);
        let selection_color = color_from(style.visuals.selection.bg_fill);

        styles.theme = *theme;
        styles.svg_background_color = svg_background_color;
        styles.selection_color = selection_color;
        styles.svg_color = match theme {
            Theme::Dark => DEFAULT_SVG_COLOR,
            Theme::Light => Color::BLACK,
        };
        *clear_color = ClearColor(svg_background_color);
    }
}

fn init_clear_color(mut clear_color: ResMut<ClearColor>, styles: Res<Styles>) {
    *clear_color = ClearColor(styles.svg_background_color);
}

fn color_from(color: Color32) -> Color {
    Color::rgba_u8(color.r(), color.g(), color.b(), color.a())
}
