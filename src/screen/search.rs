use crate::components::image_container::ImageContainer;
use crate::components::tag_selector;
use crate::components::tag_selector::TagSelector;
use crate::config::get_settings;
use crate::dtos::image_dto::ImageDTO;
use crate::dtos::tag_dto::TagDTO;
use crate::models::filter::{Filter, SortOrder};
use crate::services::clipboard_service::copy_image_to_clipboard;
use crate::services::toast_service::{push_error, push_success};
use crate::services::{file_service, image_service, tag_service};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::image::{Handle, viewer};
use iced::widget::{
    Button, Column, Container, PickList, Row, Scrollable, Space, Text, TextInput, button, stack,
};
use iced::{Alignment, Element, Length, Task};
use iced_font_awesome::fa_icon;
use iced_modern_theme::Modern;
use image::DynamicImage;
use log::{error, info};
use std::path::Path;
use std::time::Duration;

pub enum Action {
    None,
    Run(Task<Message>),
    NavigateToUpdate(ImageDTO),
    NavigatorToRegister(Option<DynamicImage>),
}

#[derive(Debug, Clone)]
pub enum Message {
    TagSelectorMessage(tag_selector::Message),
    QueryChanged(String),
    DelayedQuery(String),
    SearchButtonPressed,
    RequestImages,
    PushContainer(Vec<ImageDTO>, u64, u64),
    OpenImage(i64),
    OpenLocalImage(i64),
    DeleteImage(i64),
    CopyImage(String),
    TagsLoaded(Vec<TagDTO>),
    GoToPage(u64),
    Update(ImageDTO),
    ClosePreview,
    NavigateToRegister,
    SortOrderChanged(SortOrder),
    ImagePasted(DynamicImage),
    NoOps,
}

pub struct Search {
    pub query: String,
    pub images: Vec<ImageContainer>,
    tag_selector: TagSelector,
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
                tag_selector: TagSelector::new(Vec::new(), false, true),
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

            Message::Update(image_dto) => {
                info!("Update image_dto: {}", image_dto.id);
                info!("Update image_dto: {:?}", image_dto.tags);
                Action::NavigateToUpdate(image_dto)
            }


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
                    |_| Message::NoOps,
                );
                Action::Run(task)
            }

            Message::CopyImage(src) => {
                let task = Task::perform(
                    async move {
                        match copy_image_to_clipboard(&src) {
                            Ok(_) => {
                                push_success(t!("message.copy.success"));
                                Message::NoOps
                            }
                            Err(e) => {
                                error!("Error copying image to clipboard: {}", e);
                                push_error(t!("message.copy.error"));
                                Message::NoOps
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
                    |_| {
                        push_success(t!("message.delete.success"));
                        Message::NoOps
                    },
                );
                Action::Run(task)
            }

            Message::PushContainer(images, current_page, total_pages) => {
                info!("Pushing {} images", images.len());
                for img in images {
                    info!("Pushing image {}", img.id);
                    info!("Tags: {:?}", img.tags.iter().map(|t| &t.name).collect::<Vec<_>>());
                    self.images.push(ImageContainer::new(img.clone()));
                }

                self.current_page = current_page;
                self.total_pages = total_pages;

                Action::None
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

            Message::TagsLoaded(tags) => {
                self.tag_selector.available = tags;
                Action::None
            }

            Message::TagSelectorMessage(msg) => {
                let _ = self.tag_selector.update(msg);
                let task = Task::perform(async move {}, |_| Message::SearchButtonPressed);
                Action::Run(task)
            }

            Message::GoToPage(page_index) => {
                let page_size = self.page_size;
                self.images.clear();
                let query = self.query.clone();
                let selected_tags = self.tag_selector.selected.clone();

                let task = Task::perform(
                    async move {
                        let mut filter = Filter::new();

                        if !query.is_empty() {
                            filter.query = query.clone();
                        }

                        if !selected_tags.is_empty() {
                            filter.tags = selected_tags.iter().map(|t| t.name.clone()).collect();
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
                let selected_tags = self.tag_selector.selected.clone();
                let selected_sort_order = self.selected_sort_order.clone();

                info!("Query: {} Tags: {:?}", query, selected_tags);

                let task = Task::perform(
                    async move {
                        let mut filter = Filter::new();

                        if !query.is_empty() {
                            filter.query = query.clone();
                        }

                        if !selected_tags.is_empty() {
                            filter.tags = selected_tags.iter().map(|t| t.name.clone()).collect();
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

            Message::NavigateToRegister => Action::NavigatorToRegister(None),
            Message::ImagePasted(dynamic_image) => { 
                info!("Image pasted in search");
                Action::NavigatorToRegister(Some(dynamic_image))
            }
            _others => Action::None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let tags_view = self.tag_selector.view().map(Message::TagSelectorMessage);
        let header = Column::new()
            .spacing(8)
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
                        Button::new(
                            Text::new(t!("search.button.register")).align_x(Horizontal::Center),
                        )
                        .style(Modern::primary_button())
                        .on_press(Message::NavigateToRegister)
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
            .push(tags_view);

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
