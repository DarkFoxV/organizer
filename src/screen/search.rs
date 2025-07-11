use crate::components::image_container::ImageContainer;
use crate::components::toast::ToastKind;
use crate::config::get_settings;
use crate::models::filter::SortOrder::CreatedAsc;
use crate::models::filter::{Filter, SortOrder};
use crate::models::image_dto::ImageDTO;
use crate::services::clipboard_service::copy_image_to_clipboard;
use crate::services::{file_service, image_service, tag_service};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::image::{Handle, viewer};
use iced::widget::{
    Button, Column, Container, PickList, Row, Scrollable, Space, Text, TextInput, button, stack,
};
use iced::{Alignment, Element, Length, Task};
use iced_font_awesome::fa_icon;
use iced_modern_theme::Modern;
use log::{error, info};
use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

pub enum Action {
    None,
    Run(Task<Message>),
    NavigateToUpdate(ImageDTO),
    ShowToast {
        kind: ToastKind,
        message: String,
        duration: Option<Duration>,
    },
}

#[derive(Debug, Clone)]
pub enum Message {
    QueryChanged(String),
    DelayedQuery(String),
    SearchButtonPressed,
    RequestImages,
    PushContainer(Vec<ImageDTO>, u64, u64),
    LoadImage(ImageContainer),
    OpenImage(i64),
    OpenLocalImage(i64),
    DeleteImage(i64),
    CopyImage(String),
    Success(String),
    TagToggled(String),
    TagsLoaded(Vec<String>),
    GoToPage(u64),
    Update(i64),
    ShowToast {
        kind: ToastKind,
        message: String,
        duration: Option<Duration>,
    },
    ClosePreview,
    NavigateWithDTO(ImageDTO),
    SortOrderChanged(SortOrder),
}

pub struct Search {
    pub query: String,
    pub images: Vec<ImageContainer>,
    pub available_tags: Vec<String>,
    pub selected_tags: HashSet<String>,
    page_size: u64,
    current_page: u64,
    total_pages: u64,
    show_preview: bool,
    preview_handle: Handle,
    selected_sort_order: SortOrder,
}

impl Search {
    pub fn new() -> (Self, Task<Message>) {
        let settings = get_settings();
        let page_size = settings.config.items_per_page;
        (
            Self {
                query: String::new(),
                images: vec![],
                available_tags: vec![],
                selected_tags: HashSet::new(),
                page_size,
                current_page: 0,
                total_pages: 0,
                show_preview: false,
                preview_handle: Handle::from_path("".to_string()),
                selected_sort_order: SortOrder::CreatedDesc,
            },
            Task::batch([
                Task::perform(async { tag_service::find_all().await }, |tags| {
                    Message::TagsLoaded(tags.expect("REASON"))
                }),
                Task::perform(
                    async move {
                        let page = image_service::find_all(Filter::new(), 0, page_size)
                            .await
                            .unwrap();
                        (page.content, page.page_number, page.total_pages)
                    },
                    |(images, current_page, total_pages)| {
                        Message::PushContainer(images, current_page, total_pages)
                    },
                ),
            ]),
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ShowToast {
                kind,
                message,
                duration,
            } => Action::ShowToast {
                kind,
                message,
                duration,
            },

            Message::QueryChanged(query) => {
                self.query = query.clone();
                let task = Task::perform(
                    {
                        let query = query.clone();
                        async move {
                            tokio::time::sleep(Duration::from_millis(300)).await;
                            query
                        }
                    },
                    |query| Message::DelayedQuery(query),
                );
                Action::Run(task)
            }

            Message::DelayedQuery(query) => {
                if self.query == query {
                    let task = Task::perform(
                        async {
                            tokio::time::sleep(Duration::from_millis(300)).await;
                        },
                        |_| Message::SearchButtonPressed,
                    );
                    Action::Run(task)
                } else {
                    Action::None
                }
            }

            Message::Update(id) => {
                let img = self
                    .images
                    .iter()
                    .find(|img| img.image_dto.id == id)
                    .unwrap();
                Action::NavigateToUpdate(img.image_dto.clone())
            }

            Message::NavigateWithDTO(dto) => Action::NavigateToUpdate(dto),

            Message::OpenLocalImage(id) => {
                let img = self.images.iter().find(|img| img.id == id).unwrap();

                let path_buf = Path::new(&img.image_dto.path)
                    .parent()
                    .expect("Image path should have a parent")
                    .to_path_buf();

                let task = Task::perform(
                    async move {
                        let _ = file_service::open_in_file_explorer(&path_buf);
                    },
                    |_| Message::ShowToast {
                        kind: ToastKind::Success,
                        message: t!("message.open.success").to_string(),
                        duration: None,
                    },
                );
                Action::Run(task)
            }

            Message::CopyImage(src) => {
                let task = Task::perform(
                    async move {
                        match copy_image_to_clipboard(&src) {
                            Ok(_) => Message::ShowToast {
                                kind: ToastKind::Success,
                                message: t!("message.copy.success").to_string(),
                                duration: None,
                            },
                            Err(e) => {
                                error!("Error copying image to clipboard: {}", e);
                                Message::ShowToast {
                                    kind: ToastKind::Error,
                                    message: t!("message.copy.error").to_string(),
                                    duration: None,
                                }
                            }
                        }
                    },
                    |msg| msg,
                );

                Action::Run(task)
            }

            Message::DeleteImage(id) => {
                self.images.retain(|img| img.id != id);
                let task = Task::perform(
                    async move {
                        image_service::delete_image(id).await.unwrap();
                        let _ = file_service::delete_image(id);
                    },
                    |_| Message::Success("Imagem deletada com sucesso".to_string()),
                );
                Action::Run(task)
            }

            Message::PushContainer(images, current_page, total_pages) => {
                let mut batch: Vec<Task<Message>> = vec![];

                for img in images {
                    let task = Task::perform(async move {}, move |_| {
                        Message::LoadImage(ImageContainer::new(img.clone()))
                    });
                    batch.push(task);
                }

                self.current_page = current_page;
                self.total_pages = total_pages;

                Action::Run(Task::batch(batch))
            }

            Message::OpenImage(id) => {
                self.show_preview = true;
                for img in &self.images {
                    if img.id == id {
                        self.preview_handle = Handle::from_path(img.image_dto.path.clone());
                        break;
                    }
                }
                Action::None
            }

            Message::ClosePreview => {
                self.show_preview = false;
                Action::None
            }

            Message::LoadImage(img) => {
                info!("Imagens loaded: {}", img.image_dto.path);
                self.images.push(img);
                Action::None
            }

            Message::TagsLoaded(tags) => {
                info!("{} tags loaded", tags.len());
                self.available_tags = tags;
                Action::None
            }

            Message::TagToggled(tag) => {
                if self.selected_tags.contains(&tag) {
                    self.selected_tags.remove(&tag);
                } else {
                    self.selected_tags.insert(tag);
                }

                let task = Task::perform(async move {}, |_| Message::SearchButtonPressed);
                Action::Run(task)
            }

            Message::GoToPage(page_index) => {
                let page_size = self.page_size;
                self.images.clear();
                let query = self.query.clone();
                let selected_tags = self.selected_tags.clone();

                let task = Task::perform(
                    async move {
                        let mut filter = Filter::new();

                        if !query.is_empty() {
                            filter.query = query.clone();
                        }

                        if !selected_tags.is_empty() {
                            filter.tags = selected_tags;
                        }

                        let page = image_service::find_all(filter, page_index, page_size)
                            .await
                            .unwrap();
                        (page.content, page.page_number, page.total_pages)
                    },
                    |(images, current_page, total_pages)| {
                        Message::PushContainer(images, current_page, total_pages)
                    },
                );
                Action::Run(task)
            }

            Message::SearchButtonPressed => {
                self.images.clear();
                let page_size = self.page_size;
                let query = self.query.clone();
                let selected_tags = self.selected_tags.clone();
                let selected_sort_order = self.selected_sort_order.clone();
                use std::collections::HashSet;
                info!("Query: {} Tags: {:?}", query, selected_tags);

                let task = Task::perform(
                    async move {
                        let mut filter = Filter::new();

                        if !query.is_empty() {
                            filter.query = query.clone();
                        }

                        if !selected_tags.is_empty() {
                            filter.tags =
                                selected_tags.iter().cloned().collect::<HashSet<String>>();
                        }

                        filter.sort_order = selected_sort_order;

                        let page = image_service::find_all(filter, 0, page_size).await.unwrap();

                        (page.content, page.page_number, page.total_pages)
                    },
                    |(images, current_page, total_pages)| {
                        Message::PushContainer(images, current_page, total_pages)
                    },
                );

                Action::Run(task)
            }

            Message::SortOrderChanged(order) => {
                self.selected_sort_order = order;
                let task = Task::perform(async move {}, |_| Message::SearchButtonPressed);
                Action::Run(task)
            }

            _others => Action::None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let header = Column::new()
            .padding(10)
            .spacing(5)
            .push(
                Row::new()
                    .spacing(10)
                    .push(
                        TextInput::new(t!("search.input.description").as_ref(), &self.query)
                            .width(Length::FillPortion(6))
                            .on_input(Message::QueryChanged)
                            .on_submit(Message::SearchButtonPressed)
                            .style(Modern::search_input()),
                    )
                    .push(
                        Button::new(
                            Text::new(t!("search.button.search")).align_x(Horizontal::Center),
                        )
                        .style(Modern::primary_button())
                        .on_press(Message::SearchButtonPressed)
                        .width(Length::FillPortion(1)),
                    )
                    .push(
                        PickList::new(
                            [SortOrder::CreatedAsc, SortOrder::CreatedDesc],
                            Some(self.selected_sort_order.clone()),
                            |selected| Message::SortOrderChanged(selected),
                        )
                        .style(Modern::pick_list())
                        .placeholder("Sort by")
                        .width(Length::FillPortion(2)),
                    ),
            )
            .push(
                Row::new()
                    .spacing(5)
                    .padding(5)
                    .push(Text::new("Tags:"))
                    .extend(self.available_tags.iter().map(|tag| {
                        let selected = self.selected_tags.contains(tag);

                        let button = if selected {
                            Button::new(Text::new(tag))
                                .style(Modern::green_tinted_button())
                                .on_press(Message::TagToggled(tag.clone()))
                                .padding(5)
                        } else {
                            Button::new(Text::new(tag))
                                .style(Modern::blue_tinted_button())
                                .on_press(Message::TagToggled(tag.clone()))
                                .padding(5)
                        };

                        button.into()
                    }))
                    .wrap(),
            );

        let mut images_row = Row::new().spacing(10);

        for image in &self.images {
            images_row = images_row.push(image.view());
        }

        let images_row = images_row.wrap();

        let images_container = Scrollable::new(
            Container::new(images_row)
                .width(Length::Fill)
                .align_x(Horizontal::Center)
                .padding(10),
        );

        let mut pagination_row = Row::new().spacing(5);

        for page_index in 0..self.total_pages {
            let label = (page_index + 1).to_string();
            let is_current = page_index == self.current_page;

            let button = if is_current {
                Button::new(Text::new(label))
                    .style(Modern::primary_button())
                    .padding(5)
            } else {
                Button::new(Text::new(label))
                    .style(Modern::blue_tinted_button())
                    .padding(5)
                    .on_press(Message::GoToPage(page_index))
            };

            pagination_row = pagination_row.push(button);
        }

        let content = Column::new()
            .spacing(40)
            .padding(10)
            .push(
                Container::new(header)
                    .width(Length::Fill)
                    .align_x(Horizontal::Center),
            )
            .push(
                Container::new(images_container)
                    .style(Modern::card_container())
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .push(
                Container::new(pagination_row)
                    .width(Length::Shrink)
                    .align_x(Horizontal::Center),
            );

        let layout = Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center);

        let image_preview = if self.show_preview {
            let header: Row<_> = Row::new()
                .width(Length::Fill)
                .align_y(Vertical::Center)
                .push(Space::with_width(Length::Fill))
                .push(
                    button(
                        Container::new(fa_icon("circle-xmark").size(20.0))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center),
                    )
                    .width(Length::Fixed(30.0))
                    .height(Length::Fixed(30.0))
                    .on_press(Message::ClosePreview)
                    .style(Modern::danger_button()),
                );

            // Corpo com a imagem centralizada
            let body = Container::new(
                viewer(self.preview_handle.clone())
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center);

            let content: Column<_> = Column::new()
                .spacing(10)
                .align_x(Horizontal::Center)
                .push(header)
                .push(body);

            Container::new(content)
                .padding(20)
                .width(Length::FillPortion(8))
                .height(Length::FillPortion(8))
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .style(Modern::accent_container())
        } else {
            Container::new(Text::new(""))
                .width(Length::Fixed(0.0))
                .height(Length::Fixed(0.0))
        };

        stack![layout, image_preview,].into()
    }
}
