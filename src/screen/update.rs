use crate::components::tag_selector;
use crate::components::tag_selector::{Message as TagSelectorMessage, TagSelector};
use crate::dtos::image_dto::{ImageDTO, ImageUpdateDTO};
use crate::dtos::tag_dto::TagDTO;
use crate::services::toast_service::{push_error, push_success};
use crate::services::{image_service, tag_service};
use iced::widget::image::Handle;
use iced::widget::{
    Button, Column, Container, Image, Row, Scrollable, Space, Text, button, text_input,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, Shadow, Task};
use iced_font_awesome::fa_icon_solid;
use iced_modern_theme::Modern;
use log::{error, info};
use std::collections::HashSet;

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
    submitted: bool,
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
            submitted: false,
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
                if self.submitted {
                    return Action::None;
                }

                let image_id = self.image_dto.id;
                let task = Task::perform(
                    async move {
                        let mut update_dto = ImageUpdateDTO::default();

                        if !description.is_empty() {
                            update_dto.description = Some(description);
                        }

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

                self.submitted = true;
                Action::Run(task)
            }
            Message::NavigateToSearch => Action::GoToSearch,

            _ => Action::None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let handle = Handle::from_path(&self.image_dto.thumbnail_path);

        // Header
        let header = Container::new(
            Row::new()
                .width(Length::Fill)
                .align_y(Alignment::Center)
                .push(Space::with_width(Length::Fill))
                .push(
                    button(
                        Container::new(fa_icon_solid("xmark").size(20.0))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center),
                    )
                    .width(Length::Fixed(40.0))
                    .height(Length::Fixed(40.0))
                    .on_press(Message::NavigateToSearch)
                    .style(Modern::danger_button()),
                ),
        )
        .padding(Padding {
            top: 10.0,
            right: 22.5,
            bottom: 0.0,
            left: 22.5,
        })
        .width(Length::Fill);

        // Image section
        let image_section = Container::new(
            Column::new()
                .spacing(20)
                .push(
                    Text::new(t!("update.section.current_image"))
                        .size(20)
                        .font(iced::Font::MONOSPACE),
                )
                .push(
                    Container::new(Image::new(handle).width(300.0).height(300.0))
                        .padding(15)
                        .style(Modern::sheet_container())
                        .align_x(Alignment::Center),
                )
                .align_x(Alignment::Center),
        )
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .padding(30)
        .style(Modern::card_container())
        .width(Length::Fill);

        // Description section
        let description_section = Container::new(
            Column::new()
                .spacing(15)
                .push(
                    Row::new()
                        .spacing(10)
                        .align_y(Alignment::Center)
                        .push(fa_icon_solid("file-lines").size(20.0))
                        .push(
                            Text::new(t!("update.section.description"))
                                .size(20)
                                .font(iced::Font::MONOSPACE),
                        ),
                )
                .push(
                    text_input(t!("register_input.description").as_ref(), &self.description)
                        .style(Modern::text_input())
                        .padding(Padding::from([12, 16]))
                        .size(16)
                        .on_input(Message::DescriptionChanged),
                ),
        )
        .padding(30)
        .style(Modern::card_container())
        .width(Length::Fill);

        // Tag section
        let tags_section = Container::new(
            Column::new()
                .spacing(15)
                .push(
                    Row::new()
                        .spacing(10)
                        .align_y(Alignment::Center)
                        .push(fa_icon_solid("tags").size(20.0))
                        .push(
                            Text::new(t!("update.section.tags"))
                                .size(20)
                                .font(iced::Font::MONOSPACE),
                        ),
                )
                .push(if self.tags_loaded {
                    self.tag_selector.view().map(Message::TagSelectorMessage)
                } else {
                    Container::new(
                        Row::new()
                            .spacing(10)
                            .align_y(Alignment::Center)
                            .push(fa_icon_solid("spinner").size(16.0))
                            .push(
                                Text::new(t!("update.loading.tags"))
                                    .size(16)
                                    .color(Color::from_rgb(0.6, 0.6, 0.6)),
                            ),
                    )
                    .padding(20)
                    .style(Modern::floating_container())
                    .into()
                }),
        )
        .padding(30)
        .style(Modern::card_container())
        .width(Length::Fill);

        // Field validation
        let description_changed = self.description != self.original_description;
        let tags_changed = self.tag_selector.selected_tags() != self.image_dto.tags;
        let has_changes = description_changed || tags_changed;

        let description_valid = !self.description.trim().is_empty();
        let tags_valid = !self.tag_selector.selected.is_empty();

        let ready =
            has_changes && description_valid && tags_valid && self.tags_loaded && !self.submitted;

        // Section of changes
        let changes_status = if has_changes {
            let mut changes_list = Column::new().spacing(8);

            if description_changed {
                changes_list = changes_list.push(
                    Row::new()
                        .spacing(8)
                        .align_y(Alignment::Center)
                        .push(fa_icon_solid("circle-dot").size(12.0))
                        .push(
                            Text::new(t!("update.changes.description"))
                                .size(14)
                                .color(Color::from_rgb(0.2, 0.6, 0.8)),
                        ),
                );
            }

            if tags_changed {
                changes_list = changes_list.push(
                    Row::new()
                        .spacing(8)
                        .align_y(Alignment::Center)
                        .push(fa_icon_solid("circle-dot").size(12.0))
                        .push(
                            Text::new(t!("update.changes.tags"))
                                .size(14)
                                .color(Color::from_rgb(0.2, 0.6, 0.8)),
                        ),
                );
            }

            Container::new(
                Column::new()
                    .spacing(10)
                    .push(
                        Row::new()
                            .spacing(10)
                            .align_y(Alignment::Center)
                            .push(fa_icon_solid("exclamation-triangle").size(16.0))
                            .push(
                                Text::new(t!("update.status.changes_detected"))
                                    .size(16)
                                    .color(Color::from_rgb(0.8, 0.6, 0.2)),
                            ),
                    )
                    .push(changes_list),
            )
            .padding(20)
            .style(|_theme: &iced::Theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgb(1.0, 0.98, 0.9))),
                border: Border {
                    radius: iced::border::Radius::from(8.0),
                    color: Color::from_rgb(0.9, 0.8, 0.6),
                    width: 1.0,
                },
                shadow: Shadow::default(),
                text_color: None,
            })
            .width(Length::Fill)
        } else {
            Container::new(
                Row::new()
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .push(fa_icon_solid("check-circle").size(16.0))
                    .push(
                        Text::new(t!("update.status.no_changes"))
                            .size(16)
                            .color(Color::from_rgb(0.5, 0.5, 0.5)),
                    ),
            )
            .padding(20)
            .style(|_theme: &iced::Theme| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgb(0.97, 0.97, 0.97))),
                border: Border {
                    radius: iced::border::Radius::from(8.0),
                    color: Color::from_rgb(0.9, 0.9, 0.9),
                    width: 1.0,
                },
                shadow: Shadow::default(),
                text_color: None,
            })
            .width(Length::Fill)
        };

        // Action section
        let action_section = Container::new(
            Column::new()
                .spacing(20)
                .align_x(Alignment::Center)
                .push(changes_status)
                .push({
                    let mut button = Button::new(
                        Row::new()
                            .spacing(12)
                            .align_y(Alignment::Center)
                            .push(
                                fa_icon_solid(if self.submitted {
                                    "hourglass-half"
                                } else {
                                    "floppy-disk"
                                })
                                .size(18.0),
                            )
                            .push(
                                Text::new(if self.submitted {
                                    t!("update.button.updating")
                                } else {
                                    t!("update.button.save")
                                })
                                .size(16),
                            ),
                    )
                    .padding(Padding::from([15, 30]));

                    if ready {
                        button = button
                            .style(Modern::success_button())
                            .on_press(Message::Submit {
                                description: self.description.clone(),
                                tags: self.tag_selector.selected_tags(),
                            });
                    } else if self.submitted {
                        button = button.style(Modern::plain_button());
                    } else {
                        button = button.style(Modern::secondary_button());
                    }

                    button
                }),
        )
        .padding(30)
        .style(Modern::floating_container())
        .width(Length::Fill);

        // Main content
        let main_content = Column::new().spacing(20).push(header).push(
            Scrollable::new(
                Column::new()
                    .padding(20)
                    .spacing(20)
                    .push(image_section)
                    .push(description_section)
                    .push(tags_section)
                    .push(Space::with_height(20))
                    .push(action_section),
            )
            .width(Length::Fill)
            .height(Length::Fill),
        );

        Container::new(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
