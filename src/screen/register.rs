use crate::components::tag_selector;
use crate::components::tag_selector::TagSelector;
use crate::dtos::image_dto::ImageUpdateDTO;
use crate::dtos::tag_dto::TagDTO;
use crate::services::file_service::save_image_file_with_thumbnail;
use crate::services::thumbnail_service::{dynamic_image_to_rgba, open_and_fix_image};
use crate::services::toast_service::{push_error, push_success};
use crate::services::{image_service, tag_service};
use iced::widget::image::Handle;
use iced::widget::{
    Button, Column, Container, Image, Row, Scrollable, Space, Text, button, text_input,
};
use iced::{Alignment, Color, Element, Length, Padding, Task};
use iced_font_awesome::{fa_icon, fa_icon_solid};
use iced_modern_theme::Modern;
use image::DynamicImage;
use log::{error, info, warn};
use rfd::AsyncFileDialog;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum Message {
    OpenImagePicker,
    ImageChosen(String),
    DescriptionChanged(String),
    TagSelectorMessage(tag_selector::Message),
    TagsLoaded(Vec<TagDTO>),
    Submit {
        description: String,
        tags: HashSet<TagDTO>,
        dynamic_image: DynamicImage,
    },
    NavigateToSearch,
    ImagePasted(DynamicImage),
    NoOps,
}

pub enum Action {
    None,
    Run(Task<Message>),
    GoToSearch,
}

pub struct Register {
    dynamic_image: Option<DynamicImage>,
    image_handle: Option<Handle>,
    description: String,
    tag_selector: TagSelector,
    tags_loaded: bool,
    submitted: bool,
}

impl Register {
    pub fn new(dynamic_image: Option<DynamicImage>) -> (Self, Task<Message>) {
        let tag_selector = TagSelector::new(Vec::new(), true, true);
        let image_handle = dynamic_image.as_ref().map(|img| dynamic_image_to_rgba(img));
        (
            Self {
                dynamic_image,
                image_handle,
                description: String::new(),
                tag_selector,
                tags_loaded: false,
                submitted: false,
            },
            Task::perform(async { tag_service::find_all().await }, |tags| match tags {
                Ok(tags) => {
                    info!("Loaded {} tags", tags.len());
                    Message::TagsLoaded(tags)
                }
                Err(err) => {
                    error!("Failed to load tags: {}", err);
                    push_error("Erro ao carregar tags");
                    Message::TagsLoaded(Vec::new())
                }
            }),
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::OpenImagePicker => {
                let task = Task::perform(
                    async {
                        AsyncFileDialog::new()
                            .add_filter(
                                "Images",
                                &["png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp"],
                            )
                            .set_directory("/")
                            .pick_file()
                            .await
                    },
                    |maybe| {
                        if let Some(file) = maybe {
                            Message::ImageChosen(file.path().to_string_lossy().to_string())
                        } else {
                            Message::NoOps
                        }
                    },
                );
                Action::Run(task)
            }
            Message::ImageChosen(path) => {
                let dynamic_image = open_and_fix_image(&path).unwrap();
                self.image_handle = dynamic_image_to_rgba(&dynamic_image).into();
                self.dynamic_image = Some(dynamic_image);
                Action::None
            }
            Message::DescriptionChanged(desc) => {
                self.description = desc;
                Action::None
            }
            Message::TagsLoaded(tags) => {
                info!("Loaded {} tags", tags.len());
                self.tag_selector.available = tags;
                self.tags_loaded = true;
                Action::None
            }
            Message::TagSelectorMessage(msg) => {
                let task: Task<tag_selector::Message> = self.tag_selector.update(msg);
                let task: Task<Message> = task.map(Message::TagSelectorMessage);
                Action::Run(task)
            }
            Message::Submit {
                dynamic_image,
                description,
                tags,
            } => {
                if self.submitted {
                    warn!("Submit already in progress, ignoring duplicate request");
                    return Action::None;
                }

                let task = Task::perform(
                    async move {
                        let image_id =
                            image_service::insert_image(&description)
                                .await
                                .map_err(|err| {
                                    error!("Erro ao inserir imagem no banco: {}", err);
                                    format!("Falha ao inserir imagem: {}", err)
                                })?;

                        let (new_path, thumb_path) = save_image_file_with_thumbnail(
                            image_id,
                            dynamic_image,
                        )
                        .map_err(|err| {
                            error!("Erro ao salvar arquivo de imagem {}: {}", image_id, err);
                            format!("Falha ao salvar arquivo: {}", err)
                        })?;

                        let mut dto = ImageUpdateDTO::default();
                        dto.path = Some(new_path);
                        dto.thumbnail_path = Some(thumb_path);
                        dto.tags = Some(tags);

                        image_service::update_from_dto(image_id, dto)
                            .await
                            .map_err(|err| {
                                error!("Erro ao atualizar imagem {}: {}", image_id, err);
                                format!("Falha ao atualizar imagem: {}", err)
                            })?;

                        info!("Image {} successfully registered", image_id);
                        Ok(())
                    },
                    |result: Result<(), String>| match result {
                        Ok(_) => {
                            push_success("Imagem registrada com sucesso!");
                            Message::NavigateToSearch
                        }
                        Err(err) => {
                            error!("Erro no processo de submit: {}", err);
                            push_error("Erro ao registrar imagem");
                            Message::NoOps
                        }
                    },
                );

                self.submitted = true;
                Action::Run(task)
            }
            Message::NavigateToSearch => Action::GoToSearch,
            Message::ImagePasted(dynamic_image) => {
                info!("Image pasted from clipboard");
                self.image_handle = Some(dynamic_image_to_rgba(&dynamic_image));
                self.dynamic_image = Some(dynamic_image);
                Action::None
            }
            Message::NoOps => Action::None,
        }
    }

    pub fn view(&self) -> Element<Message> {
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

        // Upload image preview
        let preview: Element<Message> = if let Some(handle) = &self.image_handle {
            Container::new(
                Image::new(handle.clone())
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .padding(15)
            .width(300.0)
            .height(300.0)
            .style(Modern::sheet_container())
            .into()
        } else {
            Container::new(
                Column::new()
                    .spacing(15)
                    .align_x(Alignment::Center)
                    .push(fa_icon("folder-open").size(48.0))
                    .push(
                        Text::new(t!("register.tooltip.select_file"))
                            .size(16)
                            .color(Color::from_rgb(0.5, 0.5, 0.5)),
                    ),
            )
            .padding(40)
            .width(300.0)
            .height(300.0)
            .align_y(Alignment::Center)
            .align_x(Alignment::Center)
            .style(Modern::sheet_container())
            .into()
        };

        let upload_section = Container::new(
            Column::new()
                .spacing(20)
                .push(
                    Text::new(t!("register.section.image"))
                        .size(20)
                        .font(iced::Font::MONOSPACE),
                )
                .push(preview)
                .push(
                    Row::new().spacing(10).push(
                        Button::new(
                            Row::new()
                                .spacing(8)
                                .align_y(Alignment::Center)
                                .push(fa_icon_solid("folder-plus").size(16.0))
                                .push(Text::new(t!("register.button.select_file"))),
                        )
                        .style(Modern::primary_button())
                        .padding(Padding::from([12, 20]))
                        .on_press(Message::OpenImagePicker),
                    ),
                ),
        )
        .padding(30)
        .style(Modern::card_container())
        .width(Length::Fill);

        // Description section
        let description_section = Container::new(
            Column::new()
                .spacing(15)
                .push(
                    Text::new(t!("register.section.description"))
                        .size(20)
                        .font(iced::Font::MONOSPACE),
                )
                .push(
                    text_input(
                        t!("register.placeholder.description").as_ref(),
                        &self.description,
                    )
                    .style(Modern::text_input())
                    .padding(Padding::from([12, 16]))
                    .size(16)
                    .on_input(Message::DescriptionChanged),
                ),
        )
        .padding(30)
        .style(Modern::card_container())
        .width(Length::Fill);

        // Tags section
        let tags_section = Container::new(
            Column::new()
                .spacing(15)
                .push(
                    Row::new()
                        .spacing(10)
                        .align_y(Alignment::Center)
                        .push(fa_icon_solid("tags").size(24.0))
                        .push(
                            Text::new(t!("register.section.tags"))
                                .size(20)
                                .font(iced::Font::MONOSPACE),
                        ),
                )
                .push(if self.tags_loaded {
                    self.tag_selector.view().map(Message::TagSelectorMessage)
                } else {
                    Container::new(
                        Row::new().spacing(10).align_y(Alignment::Center).push(
                            Text::new(t!("register.loading.tags"))
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

        // Fields validation
        let ready = self.image_handle.is_some()
            & &!self.description.trim().is_empty()
            & &!self.tag_selector.selected.is_empty()
            & &self.dynamic_image.is_some();

        let submit_section = Container::new(
            Column::new()
                .spacing(20)
                .push(if ready {
                    Row::new()
                        .spacing(10)
                        .align_y(Alignment::Center)
                        .push(fa_icon_solid("check").size(16.0))
                        .push(
                            Text::new(t!("register.status.ready"))
                                .size(16)
                                .color(Color::from_rgb(0.2, 0.7, 0.2)),
                        )
                } else {
                    Row::new()
                        .spacing(10)
                        .align_y(Alignment::Center)
                        .push(fa_icon_solid("exclamation").size(16.0))
                        .push(
                            Text::new(t!("register.status.incomplete"))
                                .size(16)
                                .color(Color::from_rgb(0.8, 0.6, 0.2)),
                        )
                })
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
                                    t!("register.button.submitting")
                                } else {
                                    t!("register.button.submit")
                                })
                                .size(16),
                            ),
                    )
                    .padding(Padding::from([15, 30]));

                    if ready & &!self.submitted {
                        button = button
                            .style(Modern::success_button())
                            .on_press(Message::Submit {
                                dynamic_image: self.dynamic_image.as_ref().unwrap().clone(),
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
                    .push(upload_section)
                    .push(description_section)
                    .push(tags_section)
                    .push(Space::with_height(20))
                    .push(submit_section),
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
