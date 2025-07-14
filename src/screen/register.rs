use crate::components::tag_selector;
use crate::components::tag_selector::TagSelector;
use crate::dtos::image_dto::ImageUpdateDTO;
use crate::dtos::tag_dto::TagDTO;
use crate::services::file_service::save_image_file_with_thumbnail;
use crate::services::thumbnail_service::{dynamic_image_to_rgba, open_and_fix_image};
use crate::services::toast_service::{push_error, push_success};
use crate::services::{image_service, tag_service};
use iced::widget::image::Handle;
use iced::widget::{Button, Column, Container, Image, Text, text_input};
use iced::{Element, Task};
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
        let tag_selector = TagSelector::new(Vec::new(), true);
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
                self.tags_loaded = true; // Marca que as tags foram carregadas
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
        let preview: Element<Message> = if let Some(handle) = &self.image_handle {
            Image::new(handle.clone()).width(200.0).height(200.0).into()
        } else {
            Text::new(t!("register.tooltip.select_image")).into()
        };

        let file_row = Column::new()
            .spacing(10)
            .push(
                Button::new(Text::new(t!("register.button.add_image")))
                    .on_press(Message::OpenImagePicker)
                    .style(Modern::primary_button()),
            )
            .push(
                Container::new(preview)
                    .padding(10)
                    .style(Modern::accent_container()),
            );

        let tags_view = self.tag_selector.view().map(Message::TagSelectorMessage);

        let mut form = Column::new()
            .padding(20)
            .spacing(20)
            .push(file_row)
            .push(
                text_input("Descrição da imagem", &self.description)
                    .style(Modern::text_input())
                    .on_input(Message::DescriptionChanged),
            )
            .push(Text::new("Tags:"))
            .push(tags_view);

        let ready = self.image_handle.is_some()
            && !self.description.trim().is_empty()
            && !self.tag_selector.selected.is_empty()
            && self.dynamic_image.is_some();

        let mut button =
            Button::new(Text::new(t!("register.button.add_image"))).style(Modern::primary_button());

        if ready && !self.submitted {
            button = button.on_press(Message::Submit {
                dynamic_image: self.dynamic_image.as_ref().unwrap().clone(),
                description: self.description.clone(),
                tags: self.tag_selector.selected_tags(),
            });
        }

        form = form.push(button);

        form.into()
    }
}
