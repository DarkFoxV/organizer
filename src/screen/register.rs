use crate::components::tag_selector::TagSelector;
use crate::components::toast::ToastKind;
use crate::components::tag_selector;
use crate::models::image_dto::ImageUpdateDTO;
use crate::services::file_service::save_image_file_with_thumbnail;
use crate::services::{image_service, tag_service};
use iced::widget::image::Handle;
use iced::widget::{text_input, Button, Column, Container, Image, Text};
use iced::{Element, Task};
use iced_modern_theme::Modern;
use rfd::AsyncFileDialog;
use std::collections::HashSet;
use std::time::Duration;

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
    ShowToast {
        kind: ToastKind,
        message: String,
        duration: Option<Duration>,
    },
    SubmitFailed(String),
}

pub enum Action {
    None,
    Run(Task<Message>),
    GoToSearch,
    Batch(Vec<Action>),
}

pub struct Register {
    image_path: String,
    description: String,
    tag_selector: TagSelector,
}

impl Register {
    pub fn new() -> (Self, Task<Message>) {
        let tag_selector = TagSelector::new(Vec::new());
        (
            Self {
                image_path: String::new(),
                description: String::new(),
                tag_selector,
            },
            Task::perform(async { tag_service::find_all().await}, |tags| {
                Message::TagsLoaded(tags.expect("REASON"))
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
                        if let Some(handle) = maybe {
                            Message::ImageChosen(handle.path().to_string_lossy().to_string())
                        } else {
                            Message::ImageChosen(String::new())
                        }
                    },
                );
                Action::Run(task)
            }
            Message::TagsLoaded(tags) => {
                self.tag_selector.available = tags;
                Action::None
            }
            Message::TagSelectorMessage(msg) => {
                self.tag_selector.update(msg);
                Action::None
            }

            Message::ImageChosen(path) => {
                self.image_path = path;
                Action::None
            }
            Message::DescriptionChanged(desc) => {
                self.description = desc;
                Action::None
            }

            Message::Submit {
                path,
                description,
                tags,
            } => {
                let task = Task::perform(
                    async move {
                        let image_id = image_service::insert_image(&description).await.unwrap();
                        match save_image_file_with_thumbnail(image_id, &path) {
                            Ok((new_path, thumb_path)) => {
                                let mut image = ImageUpdateDTO::default();
                                image.path = Some(new_path.clone());
                                image.thumbnail_path = Some(thumb_path.clone());
                                image.tags = Some(tags.clone());
                                image_service::update_from_dto(image_id, image).await.unwrap();
                                Ok(())
                            }
                            Err(err) => Err(err),
                        }
                    },
                    |result| match result {
                        Ok(_) => Message::ShowToast {
                            kind: ToastKind::Success,
                            message: "Image registered successfully".to_string(),
                            duration: None,
                        },
                        Err(err) => Message::ShowToast {
                            kind: ToastKind::Error,
                            message: err.to_string(),
                            duration: None,
                        },
                    },
                );

                Action::Batch(vec![Action::GoToSearch, Action::Run(task)])
            }
            _ => Action::None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let preview: Element<Message> = if !self.image_path.is_empty() {
            let handle = Handle::from_path(&self.image_path);
            Image::new(handle).width(200.0).height(200.0).into()
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

        if ready {
            form = form.push(Button::new(Text::new(t!("register.button.add_image"))).on_press(Message::Submit {
                path: self.image_path.clone(),
                description: self.description.clone(),
                tags: self.tag_selector.selected_tags(),
            }));
        }

        form.into()
    }
}
