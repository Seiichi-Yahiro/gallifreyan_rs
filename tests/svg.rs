use bevy::prelude::{App, Color, Events};
use gallifreyan_lib::plugins::color_theme::{ColorTheme, DRAW_COLOR};
use gallifreyan_lib::plugins::svg::{export::SVGExportSystemParams, SVGPlugin};
use gallifreyan_lib::plugins::text_converter::components::NestingSettings;
use gallifreyan_lib::plugins::text_converter::{SetText, TextConverterPlugin};
use std::sync::mpsc::sync_channel;

trait TestApp {
    fn new_test(nesting_settings: NestingSettings) -> Self;
    fn set_text(&mut self, text: &str) -> &mut Self;
    fn assert_svg(&mut self, file: &str);
}

impl TestApp for App {
    fn new_test(nesting_settings: NestingSettings) -> Self {
        let mut app = App::new();
        let mut color_theme = ColorTheme::default();
        color_theme.insert(DRAW_COLOR, Color::BLACK, Color::BLACK);

        app.add_plugin(TextConverterPlugin)
            .insert_resource(color_theme)
            .add_plugin(SVGPlugin)
            .insert_resource(nesting_settings);

        app
    }

    fn set_text(&mut self, text: &str) -> &mut Self {
        self.world
            .resource_mut::<Events<SetText>>()
            .send(SetText(text.to_string()));

        self.update();

        self
    }

    fn assert_svg(&mut self, file: &str) {
        let (sender, receiver) = sync_channel::<String>(1);

        self.add_system(move |svg_export: SVGExportSystemParams| {
            let svg = svg_export.create_svg().unwrap();
            sender.send(svg.to_string()).unwrap();
        });

        self.update();

        let result = receiver.recv().unwrap();
        assert_eq!(result, file.replace("\r\n", "\n"));
    }
}

#[test]
fn vocal_nesting_a() {
    App::new_test(NestingSettings::All)
        .set_text("abajatatha")
        .assert_svg(include_str!("svg/abajatatha.svg"));
}

#[test]
fn vocal_nesting_e() {
    App::new_test(NestingSettings::All)
        .set_text("ebejetethe")
        .assert_svg(include_str!("svg/ebejetethe.svg"));
}

#[test]
fn vocal_nesting_i() {
    App::new_test(NestingSettings::All)
        .set_text("ibijitithi")
        .assert_svg(include_str!("svg/ibijitithi.svg"));
}

#[test]
fn vocal_nesting_o() {
    App::new_test(NestingSettings::All)
        .set_text("obojototho")
        .assert_svg(include_str!("svg/obojototho.svg"));
}

#[test]
fn vocal_nesting_u() {
    App::new_test(NestingSettings::All)
        .set_text("ubujututhu")
        .assert_svg(include_str!("svg/ubujututhu.svg"));
}

#[test]
fn consonant_deep_cut() {
    App::new_test(NestingSettings::None)
        .set_text("bchdhgf")
        .assert_svg(include_str!("svg/bchdhgf.svg"));
}

#[test]
fn consonant_inside() {
    App::new_test(NestingSettings::None)
        .set_text("jphklcnpm")
        .assert_svg(include_str!("svg/jphklcnpm.svg"));
}

#[test]
fn consonant_shallow_cut() {
    App::new_test(NestingSettings::None)
        .set_text("twhshrvws")
        .assert_svg(include_str!("svg/twhshrvws.svg"));
}

#[test]
fn consonant_on_line() {
    App::new_test(NestingSettings::None)
        .set_text("thghyzqquxng")
        .assert_svg(include_str!("svg/thghyzqquxng.svg"));
}
