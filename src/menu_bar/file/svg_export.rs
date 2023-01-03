use crate::image_types::{
    CircleChildren, Dot, Letter, Radius, Sentence, Text, Word, OUTER_CIRCLE_SIZE, SVG_SIZE,
};
use crate::svg_builder::{
    CircleBuilder, Fill, GroupBuilder, MaskBuilder, SVGBuilder, Stroke, Title,
};
use bevy::ecs::system::SystemParam;
use bevy::math::Affine2;
use bevy::prelude::*;

type SentenceQuery<'w, 's> =
    Query<'w, 's, (&'static Text, &'static Radius, &'static GlobalTransform), With<Sentence>>;

type WordQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Radius,
        &'static GlobalTransform,
        &'static CircleChildren,
    ),
    With<Word>,
>;

type LetterQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Parent,
        &'static Letter,
        &'static Radius,
        &'static GlobalTransform,
    ),
    With<Letter>,
>;

type DotQuery<'w, 's> = Query<'w, 's, (&'static Radius, &'static GlobalTransform), With<Dot>>;

#[derive(SystemParam)]
pub struct SVGQueries<'w, 's> {
    pub sentence_query: SentenceQuery<'w, 's>,
    pub word_query: WordQuery<'w, 's>,
    pub letter_query: LetterQuery<'w, 's>,
    pub dot_query: DotQuery<'w, 's>,
}

pub fn convert_to_svg(svg_queries: SVGQueries) -> SVGBuilder {
    let mut svg = SVGBuilder::new(SVG_SIZE);
    if let Ok(text) = svg_queries
        .sentence_query
        .get_single()
        .map(|(text, _, _)| text.to_string())
    {
        svg.add(Title::new(text));
    }

    let group_transform = Affine2 {
        translation: Vec2::ZERO,
        matrix2: Mat2::from_cols(Vec2::X, Vec2::NEG_Y),
    }
    .into();

    let mut group = GroupBuilder::new().with_transform(group_transform);

    convert_sentences(&svg_queries, &mut group);
    convert_words(&svg_queries, &mut group);
    convert_letters(&svg_queries, &mut group);
    convert_dots(&svg_queries, &mut group);

    svg.add(group);

    svg
}

fn convert_sentences(svg_queries: &SVGQueries, group: &mut GroupBuilder) {
    for (_, sentence_radius, sentence_transform) in svg_queries.sentence_query.iter() {
        let position = sentence_transform.translation().truncate();

        let outer_circle = CircleBuilder::new(sentence_radius.0 + OUTER_CIRCLE_SIZE)
            .with_stroke(Stroke::Black)
            .with_fill(Fill::None)
            .with_position(position);

        let inner_circle = CircleBuilder::new(sentence_radius.0)
            .with_stroke(Stroke::Black)
            .with_fill(Fill::None)
            .with_position(position);

        group.add(outer_circle);
        group.add(inner_circle);
    }
}

fn convert_words(svg_queries: &SVGQueries, group: &mut GroupBuilder) {
    for (word_entity, word_radius, word_transform, letters) in svg_queries.word_query.iter() {
        let cutting_letters = svg_queries
            .letter_query
            .iter_many(letters.iter())
            .filter(|(_, _, letter, _, _)| letter.is_cutting())
            .collect::<Vec<_>>();

        let word = CircleBuilder::new(word_radius.0)
            .with_stroke(Stroke::Black)
            .with_position(word_transform.translation().truncate());

        if cutting_letters.is_empty() {
            group.add(word.with_fill(Fill::None));
        } else {
            let id = format!("{:?}", word_entity);
            let mut mask = MaskBuilder::new(id.clone());

            let word_mask = CircleBuilder::new(word_radius.0)
                .with_stroke(Stroke::White)
                .with_fill(Fill::Black)
                .with_position(word_transform.translation().truncate());

            mask.add(word_mask);

            for (_, _, _, letter_radius, letter_transform) in cutting_letters {
                let letter_mask = CircleBuilder::new(letter_radius.0)
                    .with_stroke(Stroke::Black)
                    .with_fill(Fill::Black)
                    .with_position(letter_transform.translation().truncate());

                mask.add(letter_mask);
            }

            group.add(mask);
            group.add(word.with_fill(Fill::Black).with_mask(Some(id)));
        }
    }
}

fn convert_letters(svg_queries: &SVGQueries, group: &mut GroupBuilder) {
    for (letter_entity, parent, letter, letter_radius, letter_transform) in
        svg_queries.letter_query.iter()
    {
        if letter.is_cutting() {
            let id = format!("{:?}", letter_entity);
            let mut mask = MaskBuilder::new(id.clone());

            let letter_mask = CircleBuilder::new(letter_radius.0)
                .with_stroke(Stroke::White)
                .with_fill(Fill::Black)
                .with_position(letter_transform.translation().truncate());

            mask.add(letter_mask);

            let (word_radius, word_transform) = svg_queries
                .word_query
                .get(parent.get())
                .map(|(_, radius, word_transform, _)| (**radius, *word_transform))
                .unwrap();

            let letter = CircleBuilder::new(word_radius)
                .with_stroke(Stroke::Black)
                .with_fill(Fill::Black)
                .with_position(word_transform.translation().truncate())
                .with_mask(Some(id));

            group.add(mask);
            group.add(letter);
        } else {
            let letter = CircleBuilder::new(letter_radius.0)
                .with_stroke(Stroke::Black)
                .with_fill(Fill::None)
                .with_position(letter_transform.translation().truncate());

            group.add(letter);
        }
    }
}

fn convert_dots(svg_queries: &SVGQueries, group: &mut GroupBuilder) {
    for (dot_radius, dot_transform) in svg_queries.dot_query.iter() {
        let dot = CircleBuilder::new(dot_radius.0)
            .with_stroke(Stroke::Black)
            .with_fill(Fill::Black)
            .with_position(dot_transform.translation().truncate());

        group.add(dot);
    }
}
