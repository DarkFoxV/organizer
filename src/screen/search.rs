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
    Button, Column, Container, PickList, Row, Scrollable, Space, Text, TextInput, button,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Task, Theme};
use iced_font_awesome::{fa_icon, fa_icon_solid};
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
    DelayedQuery(String, u64),
    SearchButtonPressed,
    RequestImages,
    PushContainer(Vec<ImageDTO>, u64, u64),
    OpenImage(i64),
    OpenLocalImage(i64),
    DeleteImage(ImageDTO),
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
    current_search_id: u64,
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
                current_search_id: 0,
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

                self.current_search_id += 1;
                let search_id = self.current_search_id;

                let task = Task::perform(
                    {
                        let query = query;
                        async move {
                            tokio::time::sleep(Duration::from_millis(300)).await;
                            (query, search_id)
                        }
                    },
                    |(query, search_id)| Message::DelayedQuery(query, search_id),
                );
                Action::Run(task)
            }

            Message::DelayedQuery(query, search_id) => {
                if self.query == query && self.current_search_id == search_id {
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

            Message::DeleteImage(dto) => {
                self.images.retain(|img| img.id != dto.id);
                let task = Task::perform(
                    async move {
                        file_service::delete_image_by_path(&dto.path).await.unwrap();
                        image_service::delete_image(dto.id).await.unwrap();
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
                    info!(
                        "Tags: {:?}",
                        img.tags.iter().map(|t| &t.name).collect::<Vec<_>>()
                    );
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
                self.preview_handle = Handle::from_path("".to_string());
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
        let tags_view = Container::new(self.tag_selector.view().map(Message::TagSelectorMessage))
            .width(Length::Fill)
            .padding(10)
            .style(Modern::card_container());

        let search_bar = Container::new(
            Row::new()
                .spacing(15)
                .push(
                    Container::new(
                        TextInput::new(t!("search.input.description").as_ref(), &self.query)
                            .on_input(Message::QueryChanged)
                            .on_submit(Message::SearchButtonPressed)
                            .style(Modern::search_input())
                            .padding([12, 16])
                            .size(16),
                    )
                    .width(Length::FillPortion(5)),
                )
                .push(
                    Button::new(
                        Container::new(
                            Row::new()
                                .spacing(8)
                                .align_y(Alignment::Center)
                                .push(fa_icon_solid("magnifying-glass").size(18.0))
                                .push(Text::new(t!("search.button.search")).size(16)),
                        )
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                    )
                    .style(Modern::primary_button())
                    .on_press(Message::SearchButtonPressed)
                    .width(Length::FillPortion(2))
                    .padding([12, 20]),
                )
                .push(
                    Button::new(
                        Container::new(
                            Row::new()
                                .spacing(8)
                                .align_y(Alignment::Center)
                                .push(fa_icon_solid("plus").size(18.0))
                                .push(Text::new(t!("search.button.register")).size(16)),
                        )
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                    )
                    .style(Modern::success_button())
                    .on_press(Message::NavigateToRegister)
                    .width(Length::FillPortion(2))
                    .padding([12, 20]),
                )
                .push(
                    Container::new(
                        PickList::new(
                            [SortOrder::CreatedAsc, SortOrder::CreatedDesc],
                            Some(self.selected_sort_order.clone()),
                            |selected| Message::SortOrderChanged(selected),
                        )
                        .style(Modern::pick_list())
                        .padding([12, 16])
                        .text_size(16),
                    )
                    .width(Length::FillPortion(1)),
                ),
        )
        .width(Length::Fill)
        .padding(20)
        .style(Modern::card_container());

        // Header
        let header = Column::new().spacing(20).push(search_bar).push(tags_view);

        // Image grid
        let mut images_row = Row::new().spacing(20);
        for image in &self.images {
            images_row = images_row.push(image.view());
        }

        let images_grid = if self.images.is_empty() {
            let column = Column::new()
                .spacing(20)
                .align_x(Alignment::Center)
                .push(Container::new(fa_icon("image").size(64.0)))
                .push(
                    Text::new("No images found")
                        .size(18)
                        .style(Modern::secondary_text()),
                )
                .push(
                    Text::new("Try adjusting your search criteria")
                        .size(14)
                        .style(Modern::secondary_text()),
                );

            Container::new(column)
                .width(Length::Fill)
                .height(Length::Fixed(300.0))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
        } else {
            Container::new(Scrollable::new(
                Container::new(images_row.wrap())
                    .width(Length::Fill)
                    .align_x(Horizontal::Center)
                    .padding(20),
            ))
            .width(Length::Fill)
            .height(Length::Fill)
        };

        let images_container = Container::new(images_grid)
            .style(Modern::card_container())
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20);

        // Pagination
        let pagination = if self.total_pages > 1 {
            let mut pagination_row = Row::new().spacing(8).align_y(Alignment::Center);

            if self.current_page > 0 {
                pagination_row = pagination_row.push(
                    Button::new(
                        Container::new(
                            Row::new()
                                .spacing(6)
                                .align_y(Alignment::Center)
                                .push(fa_icon_solid("chevron-left").size(14.0))
                                .push(Text::new(t!("search.button.previous")).size(14)),
                        )
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                    )
                    .style(Modern::secondary_button())
                    .on_press(Message::GoToPage(self.current_page - 1))
                    .padding([8, 12]),
                );
            }

            let start_page = if self.current_page > 2 {
                self.current_page - 2
            } else {
                0
            };
            let end_page = std::cmp::min(start_page + 5, self.total_pages);

            if start_page > 0 {
                pagination_row = pagination_row.push(
                    Button::new(Text::new("1").size(14))
                        .style(Modern::blue_tinted_button())
                        .on_press(Message::GoToPage(0))
                        .padding([8, 12]),
                );
                if start_page > 1 {
                    pagination_row = pagination_row
                        .push(Text::new("...").size(14).style(Modern::secondary_text()));
                }
            }

            for page_index in start_page..end_page {
                let label = (page_index + 1).to_string();
                let is_current = page_index == self.current_page;

                let button = if is_current {
                    Button::new(Text::new(label).size(14))
                        .style(Modern::primary_button())
                        .padding([8, 12])
                } else {
                    Button::new(Text::new(label).size(14))
                        .style(Modern::blue_tinted_button())
                        .on_press(Message::GoToPage(page_index))
                        .padding([8, 12])
                };

                pagination_row = pagination_row.push(button);
            }

            if end_page < self.total_pages {
                if end_page < self.total_pages - 1 {
                    pagination_row = pagination_row
                        .push(Text::new("...").size(14).style(Modern::secondary_text()));
                }
                pagination_row = pagination_row.push(
                    Button::new(Text::new(self.total_pages.to_string()).size(14))
                        .style(Modern::blue_tinted_button())
                        .on_press(Message::GoToPage(self.total_pages - 1))
                        .padding([8, 12]),
                );
            }

            if self.current_page < self.total_pages - 1 {
                pagination_row = pagination_row.push(
                    Button::new(
                        Container::new(
                            Row::new()
                                .spacing(6)
                                .align_y(Alignment::Center)
                                .push(Text::new(t!("search.button.next")).size(14))
                                .push(fa_icon_solid("chevron-right").size(14.0)),
                        )
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                    )
                    .style(Modern::secondary_button())
                    .on_press(Message::GoToPage(self.current_page + 1))
                    .padding([8, 12]),
                );
            }

            Container::new(pagination_row)
                .width(Length::Shrink)
                .align_x(Horizontal::Center)
                .padding(20)
        } else {
            Container::new(Text::new(""))
                .width(Length::Fixed(0.0))
                .height(Length::Fixed(0.0))
        };

        let content = Column::new()
            .spacing(30)
            .push(header)
            .push(images_container)
            .push(pagination);

        let layout = Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20);

        // Preview modal
        let image_preview = if self.show_preview {
            let header: Row<_> = Row::new()
                .width(Length::Fill)
                .align_y(Vertical::Center)
                .push(Space::with_width(Length::Fill))
                .push(
                    button(
                        Container::new(fa_icon_solid("xmark").size(24.0))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center),
                    )
                    .width(Length::Fixed(40.0))
                    .height(Length::Fixed(40.0))
                    .on_press(Message::ClosePreview)
                    .style(Modern::danger_button()),
                );

            let body = Container::new(
                viewer(self.preview_handle.clone())
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center);

            let modal_content: Column<_> = Column::new()
                .spacing(15)
                .align_x(Horizontal::Center)
                .push(header)
                .push(body);

            let modal = Container::new(modal_content)
                .padding(30)
                .width(Length::FillPortion(9))
                .height(Length::FillPortion(9))
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .style(|theme: &Theme| iced::widget::container::Style {
                    background: Some(Background::Color(theme.palette().background)),
                    border: Border {
                        color: Default::default(),
                        width: 0.0,
                        radius: 10.0.into(),
                    },
                    shadow: iced::Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                        offset: iced::Vector::new(0.0, 8.0),
                        blur_radius: 16.0,
                    },
                    ..Default::default()
                });

            modal
        } else {
            Container::new(Text::new(""))
                .width(Length::Fixed(0.0))
                .height(Length::Fixed(0.0))
        };

        if self.show_preview {
            image_preview.into()
        } else {
            layout.into()
        }
    }
}
