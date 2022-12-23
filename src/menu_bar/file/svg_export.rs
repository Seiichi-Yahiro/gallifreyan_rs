use crate::image_types::{
    CircleChildren, Dot, Letter, Placement, Radius, Sentence, Text, Word, OUTER_CIRCLE_SIZE,
    SVG_SIZE,
};
use crate::svg_builder::{
    AsMat3, CircleBuilder, Fill, GroupBuilder, MaskBuilder, SVGBuilder, Stroke, Title,
};
use bevy::ecs::system::SystemParam;
use bevy::math::Affine2;
use bevy::prelude::*;

type SentenceQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Text,
        &'static Radius,
        &'static Transform,
        &'static CircleChildren,
    ),
    With<Sentence>,
>;

type WordQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Radius,
        &'static Transform,
        &'static CircleChildren,
    ),
    With<Word>,
>;

type LetterQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Radius,
        &'static Transform,
        &'static Placement,
        &'static CircleChildren,
    ),
    With<Letter>,
>;

type DotQuery<'w, 's> = Query<'w, 's, (&'static Radius, &'static Transform), With<Dot>>;

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
        .map(|(text, _, _, _)| text.to_string())
    {
        svg.add(Title::new(text));
    }

    let group_transform = Affine2 {
        translation: Vec2::ZERO,
        matrix2: Mat2::from_cols(Vec2::X, Vec2::NEG_Y),
    }
    .into();

    let mut group = GroupBuilder::new().with_transform(group_transform);

    convert_sentences(svg_queries, &mut group);

    svg.add(group);

    svg
}

fn convert_sentences(svg_queries: SVGQueries, group: &mut GroupBuilder) {
    for (_, sentence_radius, sentence_transform, words) in svg_queries.sentence_query.iter() {
        let mut sentence_group =
            GroupBuilder::new().with_transform(sentence_transform.as_mat3(false));

        let outer_circle = CircleBuilder::new(sentence_radius.0 + OUTER_CIRCLE_SIZE)
            .with_stroke(Stroke::Black)
            .with_fill(Fill::None);

        let inner_circle = CircleBuilder::new(sentence_radius.0)
            .with_stroke(Stroke::Black)
            .with_fill(Fill::None);

        sentence_group.add(outer_circle);
        sentence_group.add(inner_circle);

        convert_words(&svg_queries, words, &mut sentence_group);

        group.add(sentence_group);
    }
}

fn convert_words(svg_queries: &SVGQueries, words: &[Entity], sentence_group: &mut GroupBuilder) {
    for (word_entity, word_radius, word_transform, letters) in
        svg_queries.word_query.iter_many(words.iter())
    {
        let mut word_group = GroupBuilder::new().with_transform(word_transform.as_mat3(false));

        let cutting_letters = svg_queries
            .letter_query
            .iter_many(letters.iter())
            .filter(|(_, _, _, placement, _)| {
                **placement == Placement::DeepCut || **placement == Placement::ShallowCut
            })
            .collect::<Vec<_>>();

        let word = CircleBuilder::new(word_radius.0).with_stroke(Stroke::Black);

        if cutting_letters.is_empty() {
            word_group.add(word.with_fill(Fill::None));
        } else {
            let id = format!("{:?}", word_entity);
            let mut mask = MaskBuilder::new(id.clone());

            let word_mask = CircleBuilder::new(word_radius.0)
                .with_stroke(Stroke::White)
                .with_fill(Fill::Black);

            mask.add(word_mask);

            for (_, letter_radius, letter_transform, _, _) in cutting_letters {
                let letter_mask = CircleBuilder::new(letter_radius.0)
                    .with_stroke(Stroke::Black)
                    .with_fill(Fill::Black)
                    .with_transform(letter_transform.as_mat3(false));

                mask.add(letter_mask);
            }

            word_group.add(mask);
            word_group.add(word.with_fill(Fill::Black).with_mask(Some(id)));
        }

        convert_letters(svg_queries, letters, word_radius.0, &mut word_group);

        sentence_group.add(word_group);
    }
}

fn convert_letters(
    svg_queries: &SVGQueries,
    letters: &[Entity],
    word_radius: f32,
    word_group: &mut GroupBuilder,
) {
    for (letter_entity, letter_radius, letter_transform, placement, dots) in
        svg_queries.letter_query.iter_many(letters.iter())
    {
        let mut letter_group = GroupBuilder::new().with_transform(letter_transform.as_mat3(false));

        match placement {
            Placement::Inside | Placement::OnLine | Placement::Outside => {
                let letter = CircleBuilder::new(letter_radius.0)
                    .with_stroke(Stroke::Black)
                    .with_fill(Fill::None);

                letter_group.add(letter);
            }
            Placement::DeepCut | Placement::ShallowCut => {
                let mut inverse_group =
                    GroupBuilder::new().with_transform(letter_transform.as_mat3(true));

                let id = format!("{:?}", letter_entity);
                let mut mask = MaskBuilder::new(id.clone());

                let letter_mask = CircleBuilder::new(letter_radius.0)
                    .with_stroke(Stroke::White)
                    .with_fill(Fill::Black)
                    .with_transform(letter_transform.as_mat3(false));

                mask.add(letter_mask);

                let letter = CircleBuilder::new(word_radius)
                    .with_stroke(Stroke::Black)
                    .with_fill(Fill::Black)
                    .with_mask(Some(id));

                inverse_group.add(mask);
                inverse_group.add(letter);
                letter_group.add(inverse_group);
            }
        }

        convert_dots(svg_queries, dots, &mut letter_group);

        word_group.add(letter_group);
    }
}

fn convert_dots(svg_queries: &SVGQueries, dots: &[Entity], letter_group: &mut GroupBuilder) {
    for (dot_radius, dot_transform) in svg_queries.dot_query.iter_many(dots.iter()) {
        let mut dot_group = GroupBuilder::new().with_transform(dot_transform.as_mat3(false));

        let dot = CircleBuilder::new(dot_radius.0)
            .with_stroke(Stroke::Black)
            .with_fill(Fill::Black);
        dot_group.add(dot);

        letter_group.add(dot_group);
    }
}
