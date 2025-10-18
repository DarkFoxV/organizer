use crate::components::tag_selector;
use crate::components::tag_selector::TagSelector;
use crate::dtos::image_dto::ImageUpdateDTO;
use crate::dtos::tag_dto::TagDTO;
use crate::services::file_service::{
    is_image_path, save_image_file_with_thumbnail, save_images_from_folder_with_thumbnails,
};
use crate::services::image_processor::{dynamic_image_to_rgba, open_image};
use crate::services::toast_service::{push_error, push_success};
use crate::services::{image_service, tag_service};
use iced::widget::image::Handle;
use iced::widget::{
    Button, Column, Container, Image, Row, Scrollable, Space, Text, text_input,
};
use iced::{Alignment, Color, Element, Length, Padding, Task};
use iced_font_awesome::{fa_icon, fa_icon_solid};
use iced_modern_theme::Modern;
use image::{DynamicImage, ImageFormat, ImageReader};
use log::{error, info};
use rfd::AsyncFileDialog;
use std::collections::HashSet;
use std::path::Path;
use crate::components::header::header;

#[derive(Debug, Clone)]
pub enum Message {
    OpenImagePicker,
    OpenFolderPicker,
    ImageChosen(String),
    DescriptionChanged(String),
    TagSelectorMessage(tag_selector::Message),
    TagsLoaded(HashSet<TagDTO>),
    Submit,
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
    original_format: Option<ImageFormat>,
    is_folder: bool,
    path: Option<String>,
    description: String,
    tag_selector: TagSelector,
    tags_loaded: bool,
    submitted: bool,
}

impl Register {
    pub fn new(dynamic_image: Option<DynamicImage>) -> (Self, Task<Message>) {
        let tag_selector = TagSelector::new(HashSet::new(), true, true);
        let image_handle = dynamic_image.as_ref().map(|img| dynamic_image_to_rgba(img));
        (
            Self {
                dynamic_image,
                image_handle,
                is_folder: false,
                path: None,
                original_format: None,
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
                    Message::TagsLoaded(HashSet::new())
                }
            }),
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::OpenImagePicker => Action::Run(pick_path(false)),
            Message::OpenFolderPicker => Action::Run(pick_path(true)),

            Message::ImageChosen(path) => {
                if is_image_path(&path) {
                    match open_image(&path) {
                        Ok(dynamic_image) => {
                            // Detectar formato do arquivo original
                            let format = ImageReader::open(&path)
                                .ok()
                                .and_then(|reader| reader.with_guessed_format().ok())
                                .and_then(|reader| reader.format())
                                .unwrap_or(ImageFormat::Png);

                            self.image_handle = Some(dynamic_image_to_rgba(&dynamic_image));
                            self.dynamic_image = Some(dynamic_image);
                            self.original_format = Some(format);
                            self.is_folder = false;
                            self.path = None;
                        }
                        Err(e) => {
                            error!("Failed to open image: {}", e);
                            self.dynamic_image = None;
                            self.image_handle = None;
                            self.original_format = None;
                        }
                    }
                } else {
                    info!("Chosen path is not an image, treating as folder");
                    self.is_folder = true;
                    self.path = Some(path);
                    self.dynamic_image = None;
                    self.image_handle = None;
                    self.original_format = None;
                }

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
            Message::Submit => {
                self.submitted = true;
                let original_format = self.original_format.clone().unwrap_or(ImageFormat::Png);
                let description = self.description.clone();
                let tags = self.tag_selector.selected.clone();

                if self.is_folder {
                    // Processar pasta
                    let folder_path = self.path.clone().unwrap();
                    let task = Task::perform(
                        async move {
                            let folder_path = Path::new(&folder_path);

                            // Inserir entrada principal no banco
                            let image_id = image_service::insert_image(&description)
                                .await
                                .map_err(|err| {
                                    error!("Erro ao inserir imagem no banco: {}", err);
                                    format!("Falha ao inserir imagem: {}", err)
                                })?;

                            // Processar todas as imagens da pasta
                            let saved_paths =
                                save_images_from_folder_with_thumbnails(image_id, folder_path)
                                    .map_err(|err| {
                                        error!(
                                            "Erro ao processar imagens da pasta {}: {}",
                                            folder_path.display(),
                                            err
                                        );
                                        format!("Falha ao processar imagens da pasta: {}", err)
                                    })?;

                            if saved_paths.is_empty() {
                                return Err("Nenhuma imagem válida encontrada na pasta".to_string());
                            }

                            // Usar o caminho da pasta como path principal e o primeiro thumbnail
                            let (image_dir, main_thumb_path) = &saved_paths[0];

                            let mut dto = ImageUpdateDTO::default();
                            dto.path = Some(image_dir.to_string());
                            dto.thumbnail_path = Some(main_thumb_path.clone());
                            dto.tags = Some(tags);
                            dto.is_folder = true;
                            dto.is_prepared = true;

                            image_service::update_from_dto(image_id, dto)
                                .await
                                .map_err(|err| {
                                    error!("Erro ao atualizar imagem {}: {}", image_id, err);
                                    format!("Falha ao atualizar imagem: {}", err)
                                })?;

                            info!(
                                "Processadas {} imagens da pasta para ID {}",
                                saved_paths.len(),
                                image_id
                            );
                            Ok(saved_paths.len())
                        },
                        |result: Result<usize, String>| match result {
                            Ok(count) => {
                                push_success(t!("message.register.folder.success", count = count));
                                Message::NavigateToSearch
                            }
                            Err(err) => {
                                error!("Erro no processo de submit da pasta: {}", err);
                                push_error(t!("message.register.folder.success", err = err));
                                Message::NoOps
                            }
                        },
                    );

                    Action::Run(task)
                } else {
                    // Processar imagem única
                    let dynamic_image = self.dynamic_image.clone().unwrap();
                    let task = Task::perform(
                        async move {
                            let image_id = image_service::insert_image(&description)
                                .await
                                .map_err(|err| {
                                    error!("Erro ao inserir imagem no banco: {}", err);
                                    format!("Falha ao inserir imagem: {}", err)
                                })?;

                            let (new_path, thumb_path) = save_image_file_with_thumbnail(
                                image_id,
                                dynamic_image,
                                original_format

                            )
                            .map_err(|err| {
                                error!("Erro ao salvar arquivo de imagem {}: {}", image_id, err);
                                format!("Falha ao salvar arquivo: {}", err)
                            })?;

                            let mut dto = ImageUpdateDTO::default();
                            dto.path = Some(new_path);
                            dto.thumbnail_path = Some(thumb_path);
                            dto.tags = Some(tags);
                            dto.is_prepared = true;

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
                                push_success(t!("message.register.success"));
                                Message::NavigateToSearch
                            }
                            Err(err) => {
                                error!("Erro no processo de submit: {}", err);
                                push_error(t!("message.register.error"));
                                Message::NoOps
                            }
                        },
                    );

                    Action::Run(task)
                }
            }
            Message::NavigateToSearch => Action::GoToSearch,
            Message::ImagePasted(dynamic_image) => {
                info!("Image pasted from clipboard");
                self.image_handle = Some(dynamic_image_to_rgba(&dynamic_image));
                self.dynamic_image = Some(dynamic_image);
                self.is_folder = false;
                self.path = None;
                Action::None
            }
            Message::NoOps => {
                self.submitted = false; // Reset submitted state on error
                Action::None
            }
        }
    }

    pub fn view(&'_ self) -> Element<'_, Message> {
        // Header
        let header = header(|| Message::NavigateToSearch);

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
        } else if self.is_folder {
            Container::new(
                Column::new()
                    .spacing(15)
                    .align_x(Alignment::Center)
                    .push(fa_icon("folder-open").size(48.0))
                    .push(
                        Text::new(t!("register.tooltip.selected_folder"))
                            .size(16)
                            .color(Color::from_rgb(0.5, 0.5, 0.5)),
                    )
                    .push(if let Some(path) = &self.path {
                        Text::new(
                            Path::new(path)
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy(),
                        )
                        .size(14)
                        .color(Color::from_rgb(0.3, 0.3, 0.3))
                    } else {
                        Text::new("")
                    }),
            )
            .padding(40)
            .width(300.0)
            .height(300.0)
            .align_y(Alignment::Center)
            .align_x(Alignment::Center)
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
                    Row::new()
                        .spacing(10)
                        .push(
                            Button::new(
                                Row::new()
                                    .spacing(8)
                                    .align_y(Alignment::Center)
                                    .push(fa_icon_solid("folder-plus").size(16.0))
                                    .push(Text::new(t!("register.button.select_image"))),
                            )
                            .style(Modern::primary_button())
                            .padding(Padding::from([12, 20]))
                            .on_press(Message::OpenImagePicker),
                        )
                        .push(
                            Button::new(
                                Row::new()
                                    .spacing(8)
                                    .align_y(Alignment::Center)
                                    .push(fa_icon_solid("folder-plus").size(16.0))
                                    .push(Text::new(t!("register.button.select_folder"))),
                            )
                            .style(Modern::primary_button())
                            .padding(Padding::from([12, 20]))
                            .on_press(Message::OpenFolderPicker),
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
        let ready = !self.description.trim().is_empty()
            && !self.tag_selector.selected.is_empty()
            && (self.dynamic_image.is_some() || self.is_folder);

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

                    if ready && !self.submitted {
                        button = button
                            .style(Modern::success_button())
                            .on_press(Message::Submit);
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

fn pick_path(folder: bool) -> Task<Message> {
    Task::perform(
        async move {
            let dialog = AsyncFileDialog::new().set_directory("/");

            if folder {
                dialog.pick_folder().await
            } else {
                dialog
                    .add_filter(
                        "Images",
                        &["png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp"],
                    )
                    .pick_file()
                    .await
            }
        },
        |maybe| {
            if let Some(file) = maybe {
                Message::ImageChosen(file.path().to_string_lossy().to_string())
            } else {
                Message::NoOps
            }
        },
    )
}
