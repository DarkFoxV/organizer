use crate::dtos::tag_dto::{TagDTO, TagUpdateDTO};
use crate::models::tag_color::TagColor;
use crate::services::tag_service;
use crate::services::toast_service::{push_error, push_success};
use crate::utils::capitalize_first;
use iced::widget::{Column, Container};
use iced::widget::{
    Space, button, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Task};
use iced_font_awesome::fa_icon_solid;
use iced_modern_theme::Modern;
use log::{debug, error, info};
use std::collections::{HashMap, HashSet};

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
    TagsLoaded(HashSet<TagDTO>),

    NewTagNameChanged(String),
    NewTagColorChanged(TagColor),
    CreateNewTag,
    TagCreateResult(Result<HashSet<TagDTO>, String>),
    NoOps,
}

#[derive(Debug, Default)]
pub struct ManageTags {
    pub tags: HashSet<TagDTO>,
    pub editing: HashMap<i64, TagUpdateDTO>,
    pub new_tag_name: String,
    pub new_tag_color: TagColor,
    pub btn_save: String,
    pub btn_cancel: String,
    pub btn_edit: String,
    pub btn_delete: String,
    pub tag_color_options: Vec<TagColor>,
}

impl ManageTags {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                tags: HashSet::new(),
                editing: HashMap::new(),
                new_tag_name: String::new(),
                new_tag_color: TagColor::Blue,
                btn_save: t!("manage_tags.button.save").to_string(),
                btn_cancel: t!("manage_tags.button.cancel").to_string(),
                btn_edit: t!("manage_tags.button.edit").to_string(),
                btn_delete: t!("manage_tags.button.delete").to_string(),
                tag_color_options: TagColor::all(),
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

                    let old_tag = self.tags.iter().find(|t| t.id == id).cloned();

                    if let Some(old_tag) = old_tag {

                        self.tags.remove(&old_tag);


                        let updated_tag = TagDTO {
                            id: old_tag.id,
                            name: edit.name.clone(),
                            color: edit.color.clone(),
                        };

                        self.tags.insert(updated_tag);
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
                                push_error(t!("message.manage_tags.update.error"));
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

            // Handlers para adicionar tag
            Message::NewTagNameChanged(name) => {
                self.new_tag_name = name;
                Action::None
            }

            Message::NewTagColorChanged(color) => {
                self.new_tag_color = color;
                Action::None
            }

            Message::CreateNewTag => {
                if self.new_tag_name.trim().is_empty() {
                    push_error(t!("message.tag.empty_name"));
                    return Action::None;
                }

                let name = self.new_tag_name.clone();
                let color = self.new_tag_color.clone();

                self.new_tag_name.clear();
                self.new_tag_color = TagColor::Blue;

                let task = Task::perform(
                    async move {
                        tag_service::save(&name, color)
                            .await
                            .map_err(|e| e.to_string())?;

                        tag_service::find_all().await.map_err(|e| e.to_string())
                    },
                    |result| Message::TagCreateResult(result),
                );
                Action::Run(task)
            }

            Message::TagCreateResult(result) => {
                match result {
                    Ok(tags) => {
                        info!("Tag created successfully, reloaded {} tags", tags.len());
                        self.tags = tags;
                        push_success(t!("message.tag.success"));
                    }
                    Err(err) => {
                        error!("Failed to create tag: {}", err);
                        push_error(t!("message.tag.error"));
                    }
                }
                Action::None
            }

            Message::NoOps => Action::None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut content_vec = Vec::new();

        let header = self.view_header();
        content_vec.push(header);

        content_vec.push(Space::new(0, 24).into());

        let add_tag_form = self.view_add_tag_form();
        content_vec.push(add_tag_form);

        if !self.tags.is_empty() {
            content_vec.push(Space::new(0, 32).into());
            content_vec.push(self.view_separator());
            content_vec.push(Space::new(0, 24).into());

            // Start column for table
            let mut table_column = Column::new()
                .push(self.view_table_header())
                .push(Space::new(0, 16));


            let mut elements: Vec<_> = self.tags.iter().collect();
            elements.sort_by(|a, b| a.name.cmp(&b.name));
            
            // Add tags rows
            for (i, tag) in elements.iter().enumerate() {
                table_column = table_column.push(self.view_tag(tag, i));
            }

            // Create table container
            let table_container = Container::new(table_column)
                .padding(20)
                .width(Length::Fill)
                .style(Modern::card_container());

            // Add table to content
            content_vec.push(table_container.into());
        }

        let content = column(content_vec)
            .spacing(0)
            .padding(20)
            .width(Length::Fill);

        container(scrollable(content).width(Length::Fill).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_header(&self) -> Element<Message> {
        let title = text(t!("manage_tags.title"))
            .size(32)
            .style(Modern::primary_text());

        let subtitle = text(t!("manage_tags.subtitle"))
            .size(16)
            .style(Modern::secondary_text());

        column![title, Space::new(0, 8), subtitle].spacing(0).into()
    }

    fn view_add_tag_form(&self) -> Element<Message> {
        let form_title = text(t!("manage_tags.add_form.title"))
            .size(20)
            .style(Modern::primary_text());

        let name_input = text_input(
            t!("manage_tags.input.name_placeholder").as_ref(),
            &self.new_tag_name,
        )
        .on_input(Message::NewTagNameChanged)
        .on_submit(Message::CreateNewTag)
        .padding(12)
        .size(16)
        .style(Modern::text_input())
        .width(Length::FillPortion(3));

        let color_picker = pick_list(
            self.tag_color_options.as_slice(),
            Some(self.new_tag_color.clone()),
            Message::NewTagColorChanged,
        )
        .style(Modern::pick_list())
        .width(Length::Fixed(140.0));

        let create_button = button(
            row![
                fa_icon_solid("plus").size(16.0),
                text(t!("manage_tags.button.create")).size(16)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .style(Modern::success_button())
        .on_press(Message::CreateNewTag)
        .padding(12);

        let form_controls = row![name_input, color_picker, create_button]
            .spacing(16)
            .align_y(Alignment::Center);

        let form_content = column![form_title, Space::new(0, 16), form_controls].spacing(0);

        container(form_content)
            .padding(20)
            .width(Length::Fill)
            .style(Modern::card_container())
            .into()
    }

    fn view_separator(&self) -> Element<Message> {
        container(
            container(text(""))
                .width(Length::Fill)
                .height(Length::Fixed(1.0))
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.2))),
                    border: Border::default(),
                    shadow: Shadow::default(),
                    text_color: None,
                }),
        )
        .padding([0, 0])
        .into()
    }

    fn view_table_header(&self) -> Element<Message> {
        let name_header = text(t!("manage_tags.table.name_header"))
            .size(14)
            .style(Modern::secondary_text());

        let color_header = text(t!("manage_tags.table.color_header"))
            .size(14)
            .style(Modern::secondary_text());

        let actions_header = text(t!("manage_tags.table.actions_header"))
            .size(14)
            .style(Modern::secondary_text());

        let header_row = row![
            container(name_header).width(Length::FillPortion(3)),
            container(color_header).width(Length::Fixed(140.0)),
            container(actions_header).width(Length::Fixed(200.0)),
        ]
        .spacing(20)
        .align_y(Alignment::Center);

        container(header_row).padding([0, 30]).into()
    }

    fn view_tag<'a>(&'a self, tag: &'a TagDTO, index: usize) -> Element<'a, Message> {
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
            .padding(10)
            .size(16)
            .on_submit(Message::SubmitTag(tag_id))
            .style(Modern::text_input())
            .into()
        } else {
            row![
                container(text("").size(12).style(|_theme| text::Style {
                    color: Some(self.get_color_from_tag_color(&tag.color)),
                }))
                .width(Length::Fixed(12.0))
                .height(Length::Fixed(12.0))
                .style(|_theme| container::Style {
                    background: Some(Background::Color(self.get_color_from_tag_color(&tag.color))),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 6.0.into(),
                    },
                    shadow: Shadow::default(),
                    text_color: None,
                }),
                Space::new(12, 0),
                text(capitalize_first(&tag.name))
                    .size(16)
                    .style(Modern::primary_text())
            ]
            .align_y(Alignment::Center)
            .into()
        };

        let color_el: Element<_> = if is_editing {
            pick_list(
                self.tag_color_options.as_slice(),
                Some(selected_color),
                move |c| Message::ColorChanged(tag_id, c),
            )
            .style(Modern::pick_list())
            .into()
        } else {
            text(tag.color.to_string())
                .size(14)
                .style(Modern::secondary_text())
                .into()
        };

        let actions = if is_editing {
            row![
                button(
                    row![
                        fa_icon_solid("check").size(14.0),
                        text(&self.btn_save).size(14)
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                )
                .on_press(Message::SubmitTag(tag_id))
                .style(Modern::success_button())
                .padding(8),
                button(
                    row![
                        fa_icon_solid("clock").size(14.0),
                        text(&self.btn_cancel).size(14)
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                )
                .on_press(Message::EditTag(tag_id))
                .style(Modern::danger_button())
                .padding(8),
            ]
        } else {
            row![
                button(
                    row![
                        fa_icon_solid("file-pen").size(14.0),
                        text(&self.btn_edit).size(14)
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                )
                .on_press(Message::EditTag(tag_id))
                .style(Modern::primary_button())
                .padding(8),
                button(
                    row![
                        fa_icon_solid("eraser").size(14.0),
                        text(&self.btn_delete).size(14)
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center)
                )
                .on_press(Message::DeleteTag(tag_id))
                .style(Modern::danger_button())
                .padding(8),
            ]
        }
        .spacing(8);

        let row_content = row!(
            container(name_el).width(Length::FillPortion(3)),
            container(color_el).width(Length::Fixed(140.0)),
            container(actions).width(Length::Fixed(200.0)),
        )
        .spacing(20)
        .align_y(Alignment::Center);

        let styled_container = if is_editing {
            container(row_content)
                .style(Modern::floating_container())
                .padding(16)
                .width(Length::Fill)
        } else if index % 2 == 0 {
            container(row_content)
                .style(Modern::sheet_container())
                .padding(16)
                .width(Length::Fill)
        } else {
            container(row_content)
                .style(Modern::floating_container())
                .padding(16)
                .width(Length::Fill)
        };

        container(styled_container).padding([10, 20]).into()
    }

    fn get_color_from_tag_color(&self, tag_color: &TagColor) -> Color {
        match tag_color {
            TagColor::Red => Color::from_rgb(0.9, 0.2, 0.2),
            TagColor::Blue => Color::from_rgb(0.2, 0.5, 0.9),
            TagColor::Green => Color::from_rgb(0.2, 0.7, 0.3),
            TagColor::Purple => Color::from_rgb(0.6, 0.2, 0.8),
            TagColor::Orange => Color::from_rgb(0.9, 0.5, 0.1),
            TagColor::Pink => Color::from_rgb(0.9, 0.4, 0.7),
            TagColor::Gray => Color::from_rgb(0.5, 0.5, 0.5),
            TagColor::Indigo => Color::from_rgb(0.3, 0.2, 0.7),
            TagColor::Teal => Color::from_rgb(0.2, 0.7, 0.7),
        }
    }
}
