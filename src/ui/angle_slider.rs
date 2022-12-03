use crate::math;
use bevy_egui::egui;
use bevy_egui::egui::{NumExt, WidgetInfo};
use std::ops::RangeInclusive;

pub struct AngleSlider<'a> {
    angle: &'a mut f32,
    angle_range: RangeInclusive<f32>,
}

impl<'a> egui::Widget for AngleSlider<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let size = egui::Vec2::splat(ui.spacing().slider_width);
        let mut response = ui.allocate_response(size, egui::Sense::click_and_drag());
        let rect = response.rect;

        let old_angle = *self.angle;

        if let Some(pointer_position_2d) = response.interact_pointer_pos() {
            self.calculate_new_angle(pointer_position_2d, rect);
        }

        if response.has_focus() {
            self.handle_keyboard_input(ui);
        }

        if ui.is_rect_visible(rect) {
            self.paint(ui, &response);
        }

        let new_angle = *self.angle;
        response.changed = old_angle != new_angle;
        response.widget_info(|| WidgetInfo::slider(new_angle as f64, "Angle"));
        response
    }
}

impl<'a> AngleSlider<'a> {
    pub fn new(angle: &'a mut f32, angle_range: RangeInclusive<f32>) -> Self {
        assert!(angle_range.start().is_sign_positive());
        assert!(angle_range.end().is_sign_positive());

        let mut slf = Self { angle, angle_range };

        slf.clamp_angle();
        slf
    }

    fn clamp_angle(&mut self) {
        *self.angle = math::clamp_angle(
            *self.angle,
            *self.angle_range.start(),
            *self.angle_range.end(),
        );
    }

    fn calculate_new_angle(&mut self, pointer_position_2d: egui::Pos2, rect: egui::Rect) {
        let pointer_vec = pointer_position_2d - rect.center();
        let pointer_vec = bevy::math::Vec2::new(pointer_vec.x, pointer_vec.y);
        let zero_degree_vec = bevy::math::Vec2::new(0.0, 1.0);

        let angle = pointer_vec.angle_between(zero_degree_vec).to_degrees();
        let angle = if angle < 0.0 { angle + 360.0 } else { angle };
        *self.angle = angle;
        self.clamp_angle();
    }

    fn handle_keyboard_input(&mut self, ui: &mut egui::Ui) {
        let increment: usize = [egui::Key::ArrowRight, egui::Key::ArrowUp]
            .into_iter()
            .map(|key| ui.input().num_presses(key))
            .sum();

        let decrement: usize = [egui::Key::ArrowLeft, egui::Key::ArrowDown]
            .into_iter()
            .map(|key| ui.input().num_presses(key))
            .sum();

        let step = increment as i32 - decrement as i32;

        if step != 0 {
            *self.angle += step as f32;
            self.clamp_angle();
        }
    }

    fn paint(&self, ui: &mut egui::Ui, response: &egui::Response) {
        let rect_center = response.rect.center();

        // normal slider and rect width
        let slider_width = ui.spacing().slider_width;

        let slider_rect_height = ui
            .text_style_height(&egui::TextStyle::Body)
            .at_least(ui.spacing().interact_size.y);

        // normal slider height / 2.0
        let slider_radius = ui
            .painter()
            .round_to_pixel((slider_rect_height / 4.0).at_least(2.0));

        // normal slider height
        let slider_height = 2.0 * slider_radius;

        let rail_position = rect_center;
        let rail_radius = (slider_width - slider_height) / 2.0;
        let rail_thickness = slider_height;

        ui.painter().circle_stroke(
            rail_position,
            rail_radius,
            egui::Stroke::new(rail_thickness, ui.visuals().widgets.inactive.bg_fill),
        );

        let zero_degree = bevy::math::Vec2::new(0.0, rail_radius);
        let [current_angle, start_angle, end_angle] =
            [self.angle, self.angle_range.start(), self.angle_range.end()]
                .map(|angle| -angle.to_radians())
                .map(|angle| bevy::math::Vec2::from_angle(angle).rotate(zero_degree))
                .map(|pos| egui::Pos2::from(pos.to_array()) + rect_center.to_vec2());

        if (*self.angle_range.end() - *self.angle_range.start()).abs() < 360.0 {
            let angle_range_stroke = egui::Stroke::new(1.0, ui.visuals().widgets.inactive.bg_fill);

            ui.painter()
                .line_segment([rect_center, start_angle], angle_range_stroke);
            ui.painter()
                .line_segment([rect_center, end_angle], angle_range_stroke);
        }

        let handle_position = current_angle;
        let handle_radius = slider_rect_height / 2.5;
        let handle_visuals = ui.style().interact(response);

        ui.painter().circle(
            handle_position,
            handle_radius + handle_visuals.expansion,
            handle_visuals.bg_fill,
            handle_visuals.fg_stroke,
        );
    }
}
