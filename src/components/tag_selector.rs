use iced::widget::{Button, Container, Row, Text, text_input};
use iced::{Alignment, Element, Length, Task};
use iced_font_awesome::fa_icon;
use iced_modern_theme::Modern;
use log::{info};
use std::collections::HashSet;
use crate::services::tag_service;
use crate::services::toast_service::{push_error, push_success};

#[derive(Debug, Clone)]
pub enum Message {
    ToggleTag(String),
    CreateNewTagPressed,
    NewTagNameChanged(String),
    CreateNewTag(String),
    TagCreateResult(Result<Vec<String>, String>),
    CancelNewTag,
}

#[derive(Debug, Clone)]
pub struct TagSelector {
    pub selected: HashSet<String>,
    pub available: Vec<String>,
    show_new_tag_input: bool,
    new_tag_name: String,
}

impl TagSelector {
    pub fn new(available: Vec<String>) -> Self {
        Self {
            selected: HashSet::new(),
            available,
            show_new_tag_input: false,
            new_tag_name: String::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleTag(tag) => {
                if self.selected.contains(&tag) {
                    self.selected.remove(&tag);
                } else {
                    self.selected.insert(tag);
                }
                Task::none()
            }
            Message::CreateNewTagPressed => {
                self.show_new_tag_input = true;
                Task::none()
            }
            Message::NewTagNameChanged(name) => {
                self.new_tag_name = name;
                Task::none()
            }
            Message::CreateNewTag(tag) => {
                self.show_new_tag_input = false;
                self.new_tag_name.clear();
                let tag_async = tag.clone();
                let task = Task::perform(
                    async move {
                        // 1. salva
                        tag_service::save(&tag_async)
                            .await
                            .map_err(|e| e.to_string())?;
                        // 2. carrega de novo
                        tag_service::find_all().await.map_err(|e| e.to_string())
                    },
                    |result| {
                        Message::TagCreateResult(result)
                    },
                );
                task
            }
            Message::CancelNewTag => {
                self.show_new_tag_input = false;
                self.new_tag_name.clear();
                Task::none()
            }
            Message::TagCreateResult(res) => {
                info!("Tag create result: {:#?}", res);
                match res {
                    Ok(tags) => {
                        self.available = tags;
                        push_success(t!("message.tag.success"));
                    }
                    Err(err) => {
                        info!("Error creating tag: {}", err);
                        push_error(t!("message.tag.error"));
                    }
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let mut row = Row::new().spacing(10);

        for tag in &self.available {
            let selected = self.selected.contains(tag);
            let label = Self::capitalize_first(tag);
            let button = if selected {
                Button::new(Text::new(label))
                    .style(Modern::green_tinted_button())
                    .padding(5)
                    .on_press(Message::ToggleTag(tag.clone()))
            } else {
                Button::new(Text::new(label))
                    .style(Modern::blue_tinted_button())
                    .padding(5)
                    .on_press(Message::ToggleTag(tag.clone()))
            };
            row = row.push(button);
        }

        // Inline new tag input
        row = row.push(
            Button::new(Text::new(t!("message.tag.new")))
                .style(Modern::secondary_button())
                .on_press(Message::CreateNewTagPressed),
        );

        if self.show_new_tag_input {
            let input_row = Row::new()
                .spacing(5)
                .push(
                    text_input("Type new tag", &self.new_tag_name)
                        .on_input(Message::NewTagNameChanged)
                        .on_submit(Message::CreateNewTag(self.new_tag_name.clone()))
                        .style(Modern::text_input())
                        .width(Length::FillPortion(95)),
                )
                .push(
                    Button::new(
                        Container::new(fa_icon("circle-xmark").size(20.0))
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center),
                    )
                    .style(Modern::danger_button())
                    .on_press(Message::CancelNewTag)
                    .padding(5)
                    .width(Length::FillPortion(5)),
                );

            row = row.push(input_row);
        }

        row.wrap().into()
    }

    pub fn selected_tags(&self) -> HashSet<String> {
        self.selected.iter().cloned().collect()
    }

    fn capitalize_first(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        }
    }
}
