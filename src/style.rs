use bevy::prelude::*;
use bevy_egui::egui::Color32;
use bevy_egui::EguiContext;

pub struct StylePlugin;

impl Plugin for StylePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::rgb_u8(27, 27, 27)))
            .init_resource::<Styles>()
            .add_startup_system(setup_styles)
            .add_startup_system(set_clear_color.after(setup_styles));
    }
}

pub const SVG_COLOR: Color = Color::WHITE;

#[derive(Debug, Default, Resource)]
pub struct Styles {
    pub svg_background_color: Color,
    pub svg_color: Color,
    pub selection_color: Color,
}

fn setup_styles(mut styles: ResMut<Styles>, mut egui_context: ResMut<EguiContext>) {
    let style = egui_context.ctx_mut().style();

    let svg_background_color = color_from(style.visuals.widgets.noninteractive.bg_fill);
    let selection_color = color_from(style.visuals.selection.bg_fill);

    *styles = Styles {
        svg_background_color,
        svg_color: SVG_COLOR,
        selection_color,
    }
}

fn set_clear_color(mut clear_color: ResMut<ClearColor>, styles: Res<Styles>) {
    *clear_color = ClearColor(styles.svg_background_color);
}

fn color_from(color: Color32) -> Color {
    Color::rgba_u8(color.r(), color.g(), color.b(), color.a())
}
