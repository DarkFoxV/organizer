use crate::components::tag_selector;
use crate::components::tag_selector::TagSelector;
use crate::models::image_dto::ImageUpdateDTO;
use crate::services::file_service::save_image_file_with_thumbnail;
use crate::services::thumbnail_service::open_and_fix_image;
use crate::services::toast_service::{push_error, push_success};
use crate::services::{image_service, tag_service};
use iced::widget::image::Handle;
use iced::widget::{Button, Column, Container, Image, Text, text_input};
use iced::{Element, Task};
use iced_modern_theme::Modern;
use image::{DynamicImage};
use log::{error, info};
use rfd::AsyncFileDialog;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum Message {
    OpenImagePicker,
    ImageChosen(String),
    DescriptionChanged(String),
    TagSelectorMessage(tag_selector::Message),
    TagsLoaded(Vec<String>),
    Submit {
        path: String,
        description: String,
        tags: HashSet<String>,
    },
    NavigateToSearch,
    NoOps,
}

pub enum Action {
    None,
    Run(Task<Message>),
    GoToSearch,
}

pub struct Register {
    image_path: String,
    image_handle: Option<Handle>,
    description: String,
    tag_selector: TagSelector,
    submitted: bool,
}

impl Register {
    pub fn new() -> (Self, Task<Message>) {
        let tag_selector = TagSelector::new(Vec::new());
        (
            Self {
                image_path: String::new(),
                image_handle: None,
                description: String::new(),
                tag_selector,
                submitted: false,
            },
            Task::perform(async { tag_service::find_all().await }, |tags| {
                Message::TagsLoaded(tags.expect("Failed to load tags"))
            }),
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::OpenImagePicker => {
                let task = Task::perform(
                    async {
                        AsyncFileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg"])
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
                self.image_handle = self.dynamic_image_to_rgba(open_and_fix_image(&path).unwrap()).into();
                self.image_path = path.clone();
                Action::None
            }
            Message::DescriptionChanged(desc) => {
                self.description = desc;
                Action::None
            }
            Message::TagsLoaded(tags) => {
                info!("Loaded tags: {:#?}", tags);
                self.tag_selector.available = tags;
                Action::None
            }
            Message::TagSelectorMessage(msg) => {
                let task: Task<tag_selector::Message> = self.tag_selector.update(msg);
                let task: Task<Message> = task.map(Message::TagSelectorMessage);
                Action::Run(task)
            }
            Message::Submit {
                path,
                description,
                tags,
            } => {
                let task = Task::perform(
                    async move {
                        let image_id = image_service::insert_image(&description).await
                            .map_err(|err| {
                                error!("Erro ao inserir imagem no banco: {}", err);
                                format!("Falha ao inserir imagem: {}", err)
                            })?;

                        let (new_path, thumb_path) = save_image_file_with_thumbnail(image_id, &path)
                            .map_err(|err| {
                                error!("Erro ao salvar arquivo de imagem {}: {}", image_id, err);
                                format!("Falha ao salvar arquivo: {}", err)
                            })?;

                        let mut dto = ImageUpdateDTO::default();
                        dto.path = Some(new_path);
                        dto.thumbnail_path = Some(thumb_path);
                        dto.tags = Some(tags);

                        image_service::update_from_dto(image_id, dto).await
                            .map_err(|err| {
                                error!("Erro ao atualizar imagem {}: {}", image_id, err);
                                format!("Falha ao atualizar imagem: {}", err)
                            })?;

                        Ok(())
                    },
                    |result: Result<(), String>| match result {
                        Ok(_) => {
                            push_success(t!("message.register.success"));
                            Message::NavigateToSearch
                        }
                        Err(err) => {
                            error!("Erro no processo de submit: {}", err);
                            push_error(t!("message.register.error"));
                            Message::NavigateToSearch
                        }
                    },
                );
                self.submitted = true;
                Action::Run(task)
            }
            Message::NavigateToSearch => Action::GoToSearch,
            Message::NoOps => Action::None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let preview: Element<Message> = if !self.image_path.is_empty() {
            Image::new(self.image_handle.clone().unwrap())
                .width(200.0)
                .height(200.0)
                .into()
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
                text_input(t!("register.input.description").as_ref(), &self.description)
                    .style(Modern::text_input())
                    .on_input(Message::DescriptionChanged),
            )
            .push(Text::new("Tags:"))
            .push(tags_view);

        let ready = !self.image_path.is_empty()
            && !self.description.is_empty()
            && !self.tag_selector.selected.is_empty();

        let mut button = Button::new(Text::new(t!("register.button.add_image")))
            .style(Modern::primary_button());

        if ready && !self.submitted {
            button = button.on_press(Message::Submit {
                path: self.image_path.clone(),
                description: self.description.clone(),
                tags: self.tag_selector.selected_tags(),
            });
        }

        form = form.push(button);

        form.into()
    }

    fn dynamic_image_to_rgba(&self, dynamic_image: DynamicImage) -> Handle {
        let rgba_image = dynamic_image.to_rgba8();
        let (width, height) = rgba_image.dimensions();

        let pixels = rgba_image.into_raw();

        Handle::from_rgba(width, height, pixels)
    }
}
