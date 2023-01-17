use crate::image_types::{LineSlot, Sentence, Text, SVG_SIZE};
use crate::svg::{
    CSSRule, Class, Group, SVGElement, Selector, StrokeLineCap, Style, StyleRule, Title, ToAffine2,
    SVG,
};
use bevy::ecs::query::QuerySingleError;
use bevy::ecs::system::SystemParam;
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::DrawMode;
use std::string::ToString;

const FILL_CLASS: &str = "fill";
const STROKE_CLASS: &str = "stroke";

type ComponentQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Transform,
        Option<&'static SVGElement>,
        Option<&'static Children>,
        Option<&'static DrawMode>,
    ),
    Without<LineSlot>,
>;

#[derive(SystemParam)]
pub struct SVGQueries<'w, 's> {
    sentence_query: Query<'w, 's, (Entity, &'static Text), With<Sentence>>,
    component_query: ComponentQuery<'w, 's>,
}

pub fn convert_to_svg(svg_queries: SVGQueries) -> Result<SVG, QuerySingleError> {
    svg_queries
        .sentence_query
        .get_single()
        .map(|(sentence_entity, text)| {
            let mut svg = SVG::new(SVG_SIZE);

            svg.push(Title(text.to_string()));

            let style = {
                let mut style = Style::new();

                let mut stroke_rule = StyleRule::new();
                stroke_rule
                    .selectors
                    .push(Selector::Class(STROKE_CLASS.to_string()));
                stroke_rule.rules.push(CSSRule::Stroke(Some(Color::BLACK)));
                stroke_rule.rules.push(CSSRule::Fill(None));
                stroke_rule.rules.push(CSSRule::StrokeWidth(1.0));
                stroke_rule
                    .rules
                    .push(CSSRule::StrokeLineCap(StrokeLineCap::Round));

                style.push(stroke_rule);

                let mut fill_rule = StyleRule::new();
                fill_rule
                    .selectors
                    .push(Selector::Class(FILL_CLASS.to_string()));
                fill_rule.rules.push(CSSRule::Fill(Some(Color::BLACK)));
                fill_rule.rules.push(CSSRule::Stroke(None));

                style.push(fill_rule);

                style
            };

            svg.push(style);

            let mut group = Group::new();

            // mirror along y-axis because svg uses a mirrored y-axis
            group.affine2 = Affine2 {
                translation: Vec2::ZERO,
                matrix2: Mat2::from_cols(Vec2::X, Vec2::NEG_Y),
            };

            group = build_svg(&svg_queries.component_query, [sentence_entity], group);

            svg.push(group);

            svg
        })
}

fn build_svg(
    query: &ComponentQuery,
    entities: impl IntoIterator<Item = Entity>,
    mut group: Group,
) -> Group {
    for (transform, svg_element, children, draw_mode) in query.iter_many(entities) {
        let mut local_group = Group::new();
        local_group.affine2 = transform.to_affine2();

        if let (Some(svg_element), Some(draw_mode)) = (svg_element, draw_mode) {
            let mut svg_element = svg_element.clone();

            match draw_mode {
                DrawMode::Fill(_) => svg_element.set_class(Class(FILL_CLASS.to_string())),
                DrawMode::Stroke(_) => svg_element.set_class(Class(STROKE_CLASS.to_string())),
                DrawMode::Outlined { .. } => {}
            }

            local_group.push(svg_element);
        }

        if let Some(children) = children {
            group.push(build_svg(query, children.iter().copied(), local_group));
        } else {
            group.push(local_group);
        }
    }

    group
}
