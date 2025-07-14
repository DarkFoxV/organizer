use crate::components::tag_selector;
use crate::components::tag_selector::{Message as TagSelectorMessage, TagSelector};
use crate::services::toast_service::{push_error, push_success};
use crate::services::{image_service, tag_service};
use iced::alignment::Vertical;
use iced::widget::image::Handle;
use iced::widget::{Button, Column, Container, Image, Row, Space, Text, button, text_input};
use iced::{Alignment, Element, Length, Task};
use iced_font_awesome::fa_icon;
use iced_modern_theme::Modern;
use log::{error, info};
use std::collections::HashSet;
use crate::dtos::image_dto::{ImageDTO, ImageUpdateDTO};
use crate::dtos::tag_dto::TagDTO;

pub enum Action {
    None,
    Run(Task<Message>),
    GoToSearch,
}

#[derive(Debug, Clone)]
pub enum Message {
    TagSelectorMessage(TagSelectorMessage),
    TagsLoaded(Vec<TagDTO>),
    DescriptionChanged(String),
    Submit {
        description: String,
        tags: HashSet<TagDTO>,
    },
    NavigateToSearch,
    NoOps,
}

pub struct Update {
    tag_selector: TagSelector,
    image_dto: ImageDTO,
    description: String,
    original_description: String,
    tags_loaded: bool,
}

impl Update {
    pub fn new(image_dto: ImageDTO) -> (Self, Task<Message>) {
        let description = image_dto.description.clone();
        let original_description = image_dto.description.clone();

        let tag_selector = TagSelector::new(Vec::new(), true, true);
        let update = Update {
            tag_selector,
            image_dto,
            description,
            original_description,
            tags_loaded: false,
        };

        // Carrega todas as tags disponíveis
        let task = Task::perform(
            async move {
                let all_tags = tag_service::find_all().await.unwrap_or_default();
                all_tags
            },
            |all_tags| Message::TagsLoaded(all_tags),
        );

        (update, task)
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::TagsLoaded(tags) => {
                self.tag_selector.available = tags;
                self.tag_selector.selected = self.image_dto.tags.clone();
                info!("Tags loaded from image: {:?}", self.image_dto.tags);
                info!("Tags loaded {:?}", self.tag_selector.selected);
                self.tags_loaded = true;
                Action::None
            }

            Message::TagSelectorMessage(msg) => {
                let task: Task<tag_selector::Message> = self.tag_selector.update(msg);
                let task: Task<Message> = task.map(Message::TagSelectorMessage);
                Action::Run(task)
            }

            Message::DescriptionChanged(desc) => {
                self.description = desc;
                Action::None
            }

            Message::Submit { description, tags } => {
                let image_id = self.image_dto.id;
                let task = Task::perform(
                    async move {
                        let mut update_dto = ImageUpdateDTO::default();

                        // Só atualiza se a descrição mudou
                        if !description.is_empty() {
                            update_dto.description = Some(description);
                        }

                        // Só atualiza se as tags mudaram
                        if !tags.is_empty() {
                            update_dto.tags = Some(tags);
                        }

                        image_service::update_from_dto(image_id, update_dto).await
                    },
                    |result| match result {
                        Ok(_) => {
                            push_success(t!("message.update.success"));
                            Message::NavigateToSearch
                        }
                        Err(err) => {
                            error!("Error updating image: {}", err);
                            push_error(t!("message.update.error"));
                            Message::NavigateToSearch
                        }
                    },
                );

                Action::Run(task)
            }
            Message::NavigateToSearch => Action::GoToSearch,

            _ => Action::None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let handle = Handle::from_path(&self.image_dto.thumbnail_path);

        // Header with image on the left and close button on the right
        let header_row = Row::new()
            .width(Length::Fill)
            .align_y(Vertical::Top)
            .spacing(20)
            .push(
                // Image on the left
                Column::new()
                    .spacing(10)
                    .push(Text::new(t!("update.tooltip.current_image")))
                    .push(
                        Container::new(Image::new(handle).width(200.0).height(200.0))
                            .padding(10)
                            .style(Modern::accent_container()),
                    ),
            )
            .push(Space::with_width(Length::Fill))
            .push(
                // Close button on the right (at the top)
                button(
                    Container::new(fa_icon("circle-xmark").size(20.0))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .width(Length::Fixed(30.0))
                .height(Length::Fixed(30.0))
                .on_press(Message::NavigateToSearch)
                .style(Modern::danger_button()),
            );

        let tags_view = self.tag_selector.view().map(Message::TagSelectorMessage);

        let mut form = Column::new()
            .padding(20)
            .spacing(20)
            .push(header_row)
            .push(
                text_input(t!("register_input.description").as_ref(), &self.description)
                    .style(Modern::text_input())
                    .on_input(Message::DescriptionChanged),
            )
            .push(Text::new("Tags:"))
            .push(tags_view);

        // Check for changes
        let description_changed = self.description != self.original_description;
        let tags_changed = self.tag_selector.selected_tags() != self.image_dto.tags;
        let has_changes = description_changed || tags_changed;

        // Check if required fields are filled
        let description_valid = !self.description.trim().is_empty();
        let tags_valid = !self.tag_selector.selected.is_empty();

        let ready = has_changes && description_valid && tags_valid && self.tags_loaded;

        // Action button - always visible but conditionally enabled/disabled
        let mut save_button = Button::new(Text::new(t!("update.button.save")));

        if ready {
            save_button = save_button
                .on_press(Message::Submit {
                    description: self.description.clone(),
                    tags: self.tag_selector.selected_tags(),
                })
                .style(Modern::primary_button());
        } else {
            save_button = save_button.style(Modern::secondary_button());
        }

        let button_row = Column::new().spacing(10).push(save_button);

        form = form.push(button_row);

        // Changes information
        if has_changes {
            let mut changes_info = Column::new().spacing(5);

            if description_changed {
                changes_info = changes_info.push(
                    Text::new("• ".to_owned() + t!("update.tooltip.description").as_ref())
                        .style(Modern::primary_text()),
                );
            }

            if tags_changed {
                changes_info = changes_info.push(
                    Text::new("• ".to_owned() + t!("update.tooltip.tags").as_ref())
                        .style(Modern::primary_text()),
                );
            }

            form = form.push(
                Container::new(changes_info)
                    .padding(10)
                    .style(Modern::sheet_container()),
            );
        }

        form.into()
    }
}
