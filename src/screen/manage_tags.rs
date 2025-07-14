use crate::dtos::tag_dto::{TagDTO, TagUpdateDTO};
use crate::models::tag_color::TagColor;
use crate::services::tag_service;
use crate::services::toast_service::{push_error, push_success};
use crate::utils::capitalize_first;
use iced::widget::{Text, button, column, container, pick_list, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Task};
use iced_modern_theme::Modern;
use log::{debug, error};
use std::collections::HashMap;

pub enum Action {
    None,
    Run(Task<Message>),
}

#[derive(Debug, Clone)]
pub enum Message {
    EditTag(i64),
    NameChanged(i64, String),
    ColorChanged(i64, TagColor),
    SubmitTag(i64),
    DeleteTag(i64),
    TagsLoaded(Vec<TagDTO>),
    NoOps,
}

#[derive(Debug, Default)]
pub struct ManageTags {
    pub tags: Vec<TagDTO>,
    pub editing: HashMap<i64, TagUpdateDTO>,
}

impl ManageTags {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                tags: Vec::new(),
                editing: HashMap::new(),
            },
            Task::perform(
                async move {
                    let all_tags = tag_service::find_all().await.unwrap_or_default();
                    all_tags
                },
                |all_tags| Message::TagsLoaded(all_tags),
            ),
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::EditTag(id) => {
                if self.editing.remove(&id).is_none() {
                    if let Some(tag) = self.tags.iter().find(|t| t.id == id) {
                        self.editing.insert(
                            id,
                            TagUpdateDTO {
                                name: tag.name.clone(),
                                color: tag.color.clone(),
                            },
                        );
                    }
                }
                Action::None
            }
            Message::NameChanged(id, name) => {
                if let Some(edit) = self.editing.get_mut(&id) {
                    edit.name = name;
                }
                Action::None
            }
            Message::ColorChanged(id, color) => {
                if let Some(edit) = self.editing.get_mut(&id) {
                    edit.color = color;
                }
                Action::None
            }
            Message::SubmitTag(id) => {
                if let Some(edit) = self.editing.remove(&id) {
                    if let Some(tag) = self.tags.iter_mut().find(|t| t.id == id) {
                        tag.name = edit.name.clone();
                        tag.color = edit.color.clone();
                    }

                    let task = Task::perform(
                        async move { tag_service::update_from_dto(id, edit).await },
                        move |result| match result {
                            Ok(tag) => {
                                debug!("Updated tag: {:#?}", tag);
                                push_success(t!("message.manage_tags.update.success"));
                                Message::NoOps
                            }
                            Err(err) => {
                                error!("Failed to update tag: {}", err);
                                push_error(t!("message.manage_tags.update.success"));
                                Message::NoOps
                            }
                        },
                    );
                    return Action::Run(task);
                }
                Action::None
            }

            Message::DeleteTag(id) => {
                self.tags.retain(|t| t.id != id);

                let task = Task::perform(
                    async move { tag_service::delete(id).await },
                    move |result| match result {
                        Ok(()) => {
                            push_success(t!("message.manage_tags.delete.success"));
                            Message::NoOps
                        }
                        Err(err) => {
                            error!("Failed to delete tag: {}", err);
                            push_error(t!("message.manage_tags.delete.error"));
                            Message::NoOps
                        }
                    },
                );
                Action::Run(task)
            }

            Message::TagsLoaded(tags) => {
                self.tags = tags;
                Action::None
            }
            Message::NoOps => Action::None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let rows = self
            .tags
            .iter()
            .enumerate()
            .map(|(i, tag)| self.view_tag(tag, i));

        let content = column(rows.collect::<Vec<_>>())
            .spacing(15)
            .padding(20)
            .max_width(800);

        scrollable(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_tag<'a>(&self, tag: &'a TagDTO, index: usize) -> Element<'a, Message> {
        let is_editing = self.editing.contains_key(&tag.id);
        let selected_color = self
            .editing
            .get(&tag.id)
            .map(|e| e.color.clone())
            .unwrap_or(tag.color.clone());
        let tag_id = tag.id;

        let name_el: Element<_> = if is_editing {
            let edit = self.editing[&tag.id].clone();
            let description_label = t!("manage_tags.input.description");
            text_input(
                description_label.as_ref(),
                capitalize_first(&edit.name).as_str(),
            )
            .on_input(move |s| Message::NameChanged(tag_id, s))
            .padding(8)
            .size(16)
            .on_submit(Message::SubmitTag(tag_id))
            .style(Modern::text_input())
            .into()
        } else {
            text(capitalize_first(&tag.name)).size(18).into()
        };

        let color_el: Element<_> = if is_editing {
            let options: &'static [TagColor] = Box::leak(Box::new(TagColor::all()));
            pick_list(options, Some(selected_color), move |c| {
                Message::ColorChanged(tag_id, c)
            })
            .style(Modern::pick_list())
            .into()
        } else {
            text(tag.color.to_string()).size(16).into()
        };

        let save_label = t!("manage_tags.button.save");
        let cancel_label = t!("manage_tags.button.cancel");
        let edit_label = t!("manage_tags.button.edit");
        let delete_label = t!("manage_tags.button.delete");

        let actions = if is_editing {
            row![
                button(Text::new(save_label))
                    .on_press(Message::SubmitTag(tag_id))
                    .style(Modern::success_button()),
                button(Text::new(cancel_label))
                    .on_press(Message::EditTag(tag_id))
                    .style(Modern::danger_button()),
            ]
        } else {
            row![
                button(Text::new(edit_label))
                    .on_press(Message::EditTag(tag_id))
                    .style(Modern::success_button()),
                button(Text::new(delete_label))
                    .on_press(Message::DeleteTag(tag_id))
                    .style(Modern::danger_button()),
            ]
        }
        .spacing(10);

        let row_content = row!(
            container(name_el).width(Length::FillPortion(3)),
            container(color_el).width(Length::Fixed(120.0)),
            container(actions).width(Length::Fixed(180.0)),
        )
        .spacing(20)
        .align_y(Alignment::Center);

        let styled_container = if index % 2 == 0 {
            container(row_content)
                .style(Modern::sheet_container())
                .padding(10)
        } else {
            container(row_content)
                .style(Modern::floating_container())
                .padding(10)
        };

        styled_container.into()
    }
}
