use crate::plugins::svg::{
    CSSRule, Class, Group, SVGElement, Selector, StrokeLineCap, Style, StyleRule, Title, ToAffine2,
    SVG,
};
use crate::plugins::text_converter::components::{LineSlot, Sentence, Text, SVG_SIZE};
use bevy::ecs::query::QuerySingleError;
use bevy::ecs::system::SystemParam;
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{Fill, Stroke};
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
        Option<&'static Stroke>,
        Option<&'static Fill>,
    ),
    Without<LineSlot>,
>;

#[derive(SystemParam)]
pub struct SVGExportSystemParams<'w, 's> {
    sentence_query: Query<'w, 's, (Entity, &'static Text), With<Sentence>>,
    component_query: ComponentQuery<'w, 's>,
}

impl<'w, 's> SVGExportSystemParams<'w, 's> {
    pub fn create_svg(&self) -> Result<SVG, QuerySingleError> {
        self.sentence_query
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

                group = self.convert_components_to_svg([sentence_entity], group);

                svg.push(group);

                svg
            })
    }

    fn convert_components_to_svg(
        &self,
        entities: impl IntoIterator<Item = Entity>,
        mut group: Group,
    ) -> Group {
        for (transform, svg_element, children, stroke, fill) in
            self.component_query.iter_many(entities)
        {
            let mut local_group = Group::new();
            local_group.affine2 = transform.to_affine2();

            if let Some(svg_element) = svg_element {
                let mut svg_element = svg_element.clone();

                if stroke.is_some() {
                    svg_element.set_class(Class(STROKE_CLASS.to_string()))
                }

                if fill.is_some() {
                    svg_element.set_class(Class(FILL_CLASS.to_string()))
                }

                local_group.push(svg_element);
            }

            if let Some(children) = children {
                group.push(self.convert_components_to_svg(children.iter().copied(), local_group));
            } else {
                group.push(local_group);
            }
        }

        group
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::plugins::svg::{Circle, Path, PathElement};
    use crate::plugins::text_converter::components::*;
    use std::sync::mpsc::sync_channel;

    #[test]
    fn export() {
        let mut app = App::new();

        let (sender, receiver) = sync_channel::<String>(1);

        app.add_system(move |svg_export: SVGExportSystemParams| {
            let result = svg_export.create_svg().unwrap().to_string();
            sender.send(result).unwrap();
        });

        let sentence = (
            Sentence,
            Text("bphva".to_string()),
            SVGElement::Group(Group {
                elements: vec![Circle::new(460.0).into(), Circle::new(450.0).into()],
                affine2: Default::default(),
                class: Default::default(),
            }),
            Transform::IDENTITY,
            Stroke::color(Color::BLACK),
        );

        app.world.spawn(sentence).with_children(|child_builder| {
            let mut path = Path::new();
            path.push(PathElement::MoveTo(Vec2::new(50.392555, 219.28427)));
            path.push(PathElement::Arc {
                radius: 225.0,
                large_arc: true,
                end: Vec2::new(-152.71907, -165.23282),
            });
            path.push(PathElement::MoveTo(Vec2::new(-219.45537, -49.64214)));
            path.push(PathElement::Arc {
                radius: 225.0,
                large_arc: false,
                end: Vec2::new(-50.392555, 219.28427),
            });

            let word = (
                SVGElement::Path(path),
                Transform::IDENTITY,
                Stroke::color(Color::BLACK),
            );

            child_builder.spawn(word).with_children(|child_builder| {
                let mut b_path = Path::new();
                b_path.push(PathElement::MoveTo(Vec2::new(50.392555, 44.90927)));
                b_path.push(PathElement::Arc {
                    radius: 67.5,
                    large_arc: true,
                    end: Vec2::new(-50.392555, 44.90927),
                });

                let b = (
                    SVGElement::Path(b_path),
                    Transform::from_xyz(0.0, -174.375, 0.0),
                    Stroke::color(Color::BLACK),
                );

                child_builder.spawn(b);

                let ph = (
                    SVGElement::Circle(Circle::new(67.5)),
                    Transform {
                        translation: Vec3::new(107.17064, 61.875015, 0.0),
                        rotation: Quat::from_xyzw(0.0, 0.0, 0.86602545, 0.49999997),
                        scale: Vec3::ONE,
                    },
                    Stroke::color(Color::BLACK),
                );

                child_builder.spawn(ph).with_children(|child_builder| {
                    let dot = (
                        SVGElement::Circle(Circle::new(6.75)),
                        Transform::from_xyz(-0.000005015882, 57.375, 0.0),
                        Fill::color(Color::BLACK),
                    );

                    child_builder.spawn(dot);
                });

                let mut v_path = Path::new();
                v_path.push(PathElement::MoveTo(Vec2::new(66.736305, -10.125015)));
                v_path.push(PathElement::Arc {
                    radius: 67.5,
                    large_arc: false,
                    end: Vec2::new(-66.73631, -10.125019),
                });

                let v = (
                    SVGElement::Path(v_path),
                    Transform {
                        translation: Vec3::new(-194.85574, 112.499985, 0.0),
                        rotation: Quat::from_xyzw(0.0, 0.0, 0.8660254, -0.50000006),
                        scale: Vec3::ONE,
                    },
                    Stroke::color(Color::BLACK),
                );

                child_builder.spawn(v).with_children(|child_builder| {
                    let position_correction = Transform::from_xyz(0.0, 225.0, 0.0);

                    child_builder
                        .spawn(position_correction)
                        .with_children(|child_builder| {
                            let a = (
                                SVGElement::Circle(Circle::new(27.0)),
                                Transform::from_xyz(0.0, -265.5, 0.0),
                                Stroke::color(Color::BLACK),
                            );

                            child_builder.spawn(a);
                        });
                });
            });
        });

        app.update();

        let result = receiver.recv().unwrap();
        let expected = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg
  xmlns="http://www.w3.org/2000/svg"
  xmlns:xlink="http://www.w3.org/1999/xlink"
  viewBox="-500 -500 1000 1000"
>
    <title>bphva</title>
    <style>
        .stroke {
            stroke: rgb(0, 0, 0);
            fill: none;
            stroke-width: 1;
            stroke-linecap: round;
        }
        .fill {
            fill: rgb(0, 0, 0);
            stroke: none;
        }
    </style>
    <g transform="matrix(1 0 0 -1 0 0)">
        <g transform="matrix(1 0 0 1 0 0)">
            <g transform="matrix(1 0 0 1 0 0)" class="stroke">
                <circle cx="0" cy="0" r="460"/>
                <circle cx="0" cy="0" r="450"/>
            </g>
            <g transform="matrix(1 0 0 1 0 0)">
                <path d="M 50.392555 -219.28427 A 225 225 0 1 1 -152.71907 165.23282 M -219.45537 49.64214 A 225 225 0 0 1 -50.392555 -219.28427" class="stroke"/>
                <g transform="matrix(1 0 0 1 0 -174.375)">
                    <path d="M 50.392555 -44.90927 A 67.5 67.5 0 1 1 -50.392555 -44.90927" class="stroke"/>
                </g>
                <g transform="matrix(-0.5000001 0.8660254 -0.8660254 -0.5000001 107.17064 61.875015)">
                    <circle cx="0" cy="0" r="67.5" class="stroke"/>
                    <g transform="matrix(1 0 0 1 -0.000005015882 57.375)">
                        <circle cx="0" cy="0" r="6.75" class="fill"/>
                    </g>
                </g>
                <g transform="matrix(-0.5 -0.8660255 0.8660255 -0.5 -194.85574 112.499985)">
                    <path d="M 66.736305 10.125015 A 67.5 67.5 0 0 1 -66.73631 10.125019" class="stroke"/>
                    <g transform="matrix(1 0 0 1 0 225)">
                        <g transform="matrix(1 0 0 1 0 -265.5)">
                            <circle cx="0" cy="0" r="27" class="stroke"/>
                        </g>
                    </g>
                </g>
            </g>
        </g>
    </g>
</svg>"#;

        assert_eq!(result, expected);
    }
}
