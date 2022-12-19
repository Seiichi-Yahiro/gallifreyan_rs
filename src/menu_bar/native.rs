use crate::event_set::SendEvent;
use crate::image_types::{
    CircleChildren, Dot, Letter, LineSlot, Placement, Radius, Sentence, Word, SVG_SIZE,
};
use crate::svg_builder::{
    AsMat3, CircleBuilder, Fill, GroupBuilder, MaskBuilder, SVGBuilder, Stroke,
};
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, IoTaskPool};
use futures::channel::oneshot;
use std::path::PathBuf;

pub struct NativePlugin;

impl Plugin for NativePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FileHandles>()
            .init_resource::<FileHandleReceiver>()
            .add_system(handle_file_handle_action_event)
            .add_system(receive_file_handle.after(handle_file_handle_action_event))
            .add_system(handle_save_event.after(receive_file_handle))
            .add_system(handle_export_event);
    }
}

#[derive(Default, Resource)]
pub struct FileHandles {
    ron: Option<PathBuf>,
    svg: Option<PathBuf>,
}

impl FileHandles {
    pub fn has_ron(&self) -> bool {
        self.ron.is_some()
    }
}

type FileHandleReceiverType = (PathBuf, super::FileHandleAction);

#[derive(Default, Resource)]
struct FileHandleReceiver(Option<oneshot::Receiver<FileHandleReceiverType>>);

const RON: &str = "Rusty Object Notation";
const RON_EXTENSIONS: &[&str] = &["ron", "txt"];

const SVG: &str = "Scalable Vector Graphics";
const SVG_EXTENSIONS: &[&str] = &["svg"];

fn handle_file_handle_action_event(
    mut events: EventReader<super::FileHandleAction>,
    mut file_handle_receiver: ResMut<FileHandleReceiver>,
) {
    if let Some(&action) = events.iter().last() {
        let (sender, receiver) = oneshot::channel::<FileHandleReceiverType>();

        let task = async move {
            let file_dialog = rfd::FileDialog::new();

            let path_buffer = match action {
                super::FileHandleAction::Open => {
                    file_dialog.add_filter(RON, RON_EXTENSIONS).pick_file()
                }
                super::FileHandleAction::Save => {
                    file_dialog.add_filter(RON, RON_EXTENSIONS).save_file()
                }
                super::FileHandleAction::Export => {
                    file_dialog.add_filter(SVG, SVG_EXTENSIONS).save_file()
                }
            };

            if let Some(path_buffer) = path_buffer {
                if sender.send((path_buffer, action)).is_err() {
                    error!("Couldn't send path buffer from open event because receiver was already closed!");
                }
            }
        };

        AsyncComputeTaskPool::get().spawn(task).detach();
        file_handle_receiver.0 = Some(receiver);
    }
}

fn receive_file_handle(
    mut file_handle_receiver: ResMut<FileHandleReceiver>,
    mut file_handles: ResMut<FileHandles>,
    mut file_actions: super::FileActions,
) {
    if let Some(mut receiver) = file_handle_receiver.0.take() {
        match receiver.try_recv() {
            Ok(Some((path_buffer, action))) => match action {
                super::FileHandleAction::Open => {
                    file_handles.ron = Some(path_buffer);
                    file_actions.dispatch(super::Load);
                }
                super::FileHandleAction::Save => {
                    file_handles.ron = Some(path_buffer);
                    file_actions.dispatch(super::Save);
                }
                super::FileHandleAction::Export => {
                    file_handles.svg = Some(path_buffer);
                    file_actions.dispatch(super::Export);
                }
            },
            Ok(None) => {
                file_handle_receiver.0 = Some(receiver);
            }
            Err(_canceled) => {}
        }
    }
}

fn handle_save_event(
    world: &World,
    mut events: EventReader<super::Save>,
    file_handles: Res<FileHandles>,
    serialize_query: Query<
        Entity,
        Or<(
            With<Sentence>,
            With<Word>,
            With<Letter>,
            With<Dot>,
            With<LineSlot>,
        )>,
    >,
) {
    if events.iter().last().is_some() {
        if let Some(path_buffer) = file_handles.ron.clone() {
            let mut builder = DynamicSceneBuilder::from_world(world);
            builder.extract_entities(serialize_query.iter());
            let scene = builder.build();

            let type_registry = world.resource::<AppTypeRegistry>();

            match scene.serialize_ron(type_registry) {
                Ok(data) => {
                    IoTaskPool::get()
                        .spawn(async move {
                            if let Err(error) = std::fs::write(path_buffer, data) {
                                let msg = format!("{}", error);

                                error!(msg);

                                rfd::MessageDialog::new()
                                    .set_title("Failed to save file")
                                    .set_description(&msg)
                                    .set_buttons(rfd::MessageButtons::Ok)
                                    .set_level(rfd::MessageLevel::Error)
                                    .show();
                            }
                        })
                        .detach();
                }
                Err(error) => {
                    error!("{}", error);
                }
            }
        }
    }
}

fn handle_export_event(
    mut events: EventReader<super::Export>,
    file_handles: Res<FileHandles>,
    sentence_query: Query<(&Radius, &Transform, &CircleChildren), With<Sentence>>,
    word_query: Query<(Entity, &Radius, &Transform, &CircleChildren), With<Word>>,
    letter_query: Query<(Entity, &Radius, &Transform, &Placement, &CircleChildren), With<Letter>>,
    dot_query: Query<(&Radius, &Transform), With<Dot>>,
) {
    if events.iter().last().is_some() {
        if let Some(path_buffer) = file_handles.svg.clone() {
            let mut svg = SVGBuilder::new(SVG_SIZE);

            let group_transform = Affine2 {
                translation: Vec2::ZERO,
                matrix2: Mat2::from_cols(Vec2::X, Vec2::NEG_Y),
            }
            .into();

            let mut group = GroupBuilder::new().with_transform(group_transform);

            for (sentence_radius, sentence_transform, words) in sentence_query.iter() {
                let mut sentence_group =
                    GroupBuilder::new().with_transform(sentence_transform.as_mat3(false));

                let sentence = CircleBuilder::new(sentence_radius.0)
                    .with_stroke(Stroke::Black)
                    .with_fill(Fill::None);
                sentence_group.add(sentence);

                for (word_entity, word_radius, word_transform, letters) in
                    word_query.iter_many(words.iter())
                {
                    let mut word_group =
                        GroupBuilder::new().with_transform(word_transform.as_mat3(false));

                    let cutting_letters = letter_query
                        .iter_many(letters.iter())
                        .filter(|(_, _, _, placement, _)| {
                            **placement == Placement::DeepCut
                                || **placement == Placement::ShallowCut
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

                    for (letter_entity, letter_radius, letter_transform, placement, dots) in
                        letter_query.iter_many(letters.iter())
                    {
                        let mut letter_group =
                            GroupBuilder::new().with_transform(letter_transform.as_mat3(false));

                        match placement {
                            Placement::Inside | Placement::OnLine | Placement::Outside => {
                                let letter = CircleBuilder::new(letter_radius.0)
                                    .with_stroke(Stroke::Black)
                                    .with_fill(Fill::None);

                                letter_group.add(letter);
                            }
                            Placement::DeepCut | Placement::ShallowCut => {
                                let mut inverse_group = GroupBuilder::new()
                                    .with_transform(letter_transform.as_mat3(true));

                                let id = format!("{:?}", letter_entity);
                                let mut mask = MaskBuilder::new(id.clone());

                                let letter_mask = CircleBuilder::new(letter_radius.0)
                                    .with_stroke(Stroke::White)
                                    .with_fill(Fill::Black)
                                    .with_transform(letter_transform.as_mat3(false));

                                mask.add(letter_mask);

                                let letter = CircleBuilder::new(word_radius.0)
                                    .with_stroke(Stroke::Black)
                                    .with_fill(Fill::Black)
                                    .with_mask(Some(id));

                                inverse_group.add(mask);
                                inverse_group.add(letter);
                                letter_group.add(inverse_group);
                            }
                        }

                        for (dot_radius, dot_transform) in dot_query.iter_many(dots.iter()) {
                            let mut dot_group =
                                GroupBuilder::new().with_transform(dot_transform.as_mat3(false));

                            let dot = CircleBuilder::new(dot_radius.0)
                                .with_stroke(Stroke::Black)
                                .with_fill(Fill::Black);
                            dot_group.add(dot);

                            letter_group.add(dot_group);
                        }

                        word_group.add(letter_group);
                    }

                    sentence_group.add(word_group);
                }

                group.add(sentence_group);
            }

            svg.add(group);

            let svg = svg.build();

            IoTaskPool::get()
                .spawn(async move {
                    std::fs::write(path_buffer, svg).unwrap();
                })
                .detach();
        }
    }
}
