use bevy::prelude::{App, Color, Events};
use gallifreyan_lib::plugins::color_theme::{ColorTheme, DRAW_COLOR};
use gallifreyan_lib::plugins::svg::{export::SVGExportSystemParams, SVGPlugin};
use gallifreyan_lib::plugins::text_converter::components::NestingSettings;
use gallifreyan_lib::plugins::text_converter::{SetText, TextConverterPlugin};
use std::sync::mpsc::sync_channel;

fn assert_svg(text: &str, file: &str, nesting_settings: NestingSettings) {
    let mut app = App::new();
    let mut color_theme = ColorTheme::default();
    color_theme.insert(DRAW_COLOR, Color::BLACK, Color::BLACK);

    app.add_plugin(TextConverterPlugin)
        .insert_resource(color_theme)
        .add_plugin(SVGPlugin)
        .insert_resource(nesting_settings);

    app.world
        .resource_mut::<Events<SetText>>()
        .send(SetText(text.to_string()));

    app.update();

    let (sender, receiver) = sync_channel::<String>(1);

    app.add_system(move |svg_export: SVGExportSystemParams| {
        let svg = svg_export.create_svg().unwrap();
        sender.send(svg.to_string()).unwrap();
    });

    app.update();

    let result = receiver.recv().unwrap();
    assert_eq!(result, file);
}

#[test]
fn vocal_nesting_a() {
    assert_svg(
        "abajatatha",
        include_str!("svg/abajatatha.svg"),
        NestingSettings::All,
    );
}

#[test]
fn vocal_nesting_e() {
    assert_svg(
        "ebejetethe",
        include_str!("svg/ebejetethe.svg"),
        NestingSettings::All,
    );
}

#[test]
fn vocal_nesting_i() {
    assert_svg(
        "ibijitithi",
        include_str!("svg/ibijitithi.svg"),
        NestingSettings::All,
    );
}

#[test]
fn vocal_nesting_o() {
    assert_svg(
        "obojototho",
        include_str!("svg/obojototho.svg"),
        NestingSettings::All,
    );
}

#[test]
fn vocal_nesting_u() {
    assert_svg(
        "ubujututhu",
        include_str!("svg/ubujututhu.svg"),
        NestingSettings::All,
    );
}

#[test]
fn consonant_deep_cut() {
    assert_svg(
        "bchdhgf",
        include_str!("svg/bchdhgf.svg"),
        NestingSettings::None,
    );
}

#[test]
fn consonant_inside() {
    assert_svg(
        "jphklcnpm",
        include_str!("svg/jphklcnpm.svg"),
        NestingSettings::None,
    );
}

#[test]
fn consonant_shallow_cut() {
    assert_svg(
        "twhshrvws",
        include_str!("svg/twhshrvws.svg"),
        NestingSettings::None,
    );
}

#[test]
fn consonant_on_line() {
    assert_svg(
        "thghyzqquxng",
        include_str!("svg/thghyzqquxng.svg"),
        NestingSettings::None,
    );
}
