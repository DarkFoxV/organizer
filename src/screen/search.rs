use crate::components::image_container::ImageContainer;
use crate::components::{empty_state, header, image_preview_modal, pagination, search_bar, tag_selector};
use crate::components::tag_selector::TagSelector;
use crate::config::{
    get_current_page, get_scroll_offset, get_search_query, get_selected_tags, get_settings,
    set_current_page, set_scroll_offset, set_search_query, set_selected_tags,
};
use crate::dtos::image_dto::ImageDTO;
use crate::dtos::tag_dto::TagDTO;
use crate::models::filter::{Filter, SortOrder};
use crate::services::clipboard_service::copy_image_to_clipboard;
use crate::services::toast_service::{push_error, push_success};
use crate::services::{file_service, image_service, tag_service};
use iced::alignment::{Horizontal};
use iced::widget::image::{Handle};
use iced::widget::{
    Column, Container, Row, Scrollable, Space, Text, TextInput, button,
    scrollable,
};
use iced::{Element, Length, Task};
use iced_modern_theme::Modern;
use image::DynamicImage;
use log::{error, info};
use std::collections::HashSet;
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
    PushContainer(Vec<ImageDTO>, u64, u64, bool),
    OpenImage(ImageDTO),
    OpenLocalImage(i64),
    DeleteImage(ImageDTO),
    DeleteImageFromFolder(ImageDTO),
    CopyImage(String),
    TagsLoaded(HashSet<TagDTO>),
    GoToPage(u64),
    Update(ImageDTO),
    ClosePreview,
    CloseFolder,
    NavigateToRegister,
    SortOrderChanged(SortOrder),
    ImagePasted(DynamicImage),
    PreviousImage,
    NextImage,
    ScrollChanged(scrollable::Viewport),
    NoOps,
}

pub struct Search {
    query: String,
    images: Vec<ImageContainer>,
    tag_selector: TagSelector,
    page_size: u64,
    current_page: u64,
    total_pages: u64,
    show_preview: bool,
    preview_handle: Handle,
    current_preview_index: usize,
    selected_sort_order: SortOrder,
    current_search_id: u64,
    folder_opened: bool,
    scroll_id: scrollable::Id,
    scroll_offset: f32,
}

impl Search {
    pub fn new() -> (Self, Task<Message>) {
        let settings = get_settings();
        let page_size = settings.config.items_per_page;
        let query = get_search_query();
        let page = get_current_page();
        let selected_tags = get_selected_tags();
        let scroll_offset = get_scroll_offset();
        let component = Self {
            query: query.clone(),
            images: Vec::with_capacity(page_size as usize),
            tag_selector: TagSelector::new(selected_tags.clone(), false, true),
            page_size,
            current_page: page,
            total_pages: 0,
            show_preview: false,
            preview_handle: Handle::from_path("".to_string()),
            current_preview_index: 0,
            selected_sort_order: SortOrder::CreatedDesc,
            current_search_id: 0,
            folder_opened: false,
            scroll_id: scrollable::Id::unique(),
            scroll_offset,
        };

        let task = Task::batch([
            Task::perform(
                async { tag_service::find_all().await },
                |result| match result {
                    Ok(tags) => Message::TagsLoaded(tags),
                    Err(_err) => {
                        push_error("Program failed to load tags");
                        Message::NoOps
                    }
                },
            ),
            Task::perform(
                async move {
                    let mut filter = Filter::new();
                    filter.query = query;
                    filter.tags = selected_tags.iter().map(|tag| tag.name.clone()).collect();

                    match image_service::find_all(filter, page, page_size).await {
                        Ok(page) => (page.content, page.page_number, page.total_pages),
                        Err(_) => (vec![], 0, 0),
                    }
                },
                |(images, current_page, total_pages)| {
                    Message::PushContainer(images, current_page, total_pages, false)
                },
            ),
        ]);

        (component, task)
    }

    // Helpers

    fn change_preview(&mut self, delta: isize) {
        if self.show_preview && !self.images.is_empty() {
            let len = self.images.len() as isize;
            // calcula o índice circular
            self.current_preview_index =
                ((self.current_preview_index as isize + delta + len) % len) as usize;

            let current_image = &self.images[self.current_preview_index];
            let path = if current_image.image_dto.is_folder {
                &current_image.image_dto.thumbnail_path
            } else {
                &current_image.image_dto.path
            };
            self.preview_handle = Handle::from_path(path.clone());
        }
    }

    fn change_scroll(&mut self) -> Task<Message> {

        let scroll_offset = self.scroll_offset;
        let scroll_id = self.scroll_id.clone();
        let task = Task::done(()).then(move |_| {
            scrollable::scroll_to(
                scroll_id.clone(),
                scrollable::AbsoluteOffset {
                    x: 0.0,
                    y: scroll_offset,
                },
            )
        });

        task
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::QueryChanged(query) => {
                self.query = query.clone();
                set_search_query(query.clone());
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

            Message::ScrollChanged(viewport) => {
                self.scroll_offset = viewport.absolute_offset().y;
                set_scroll_offset(self.scroll_offset);
                Action::None
            }

            Message::Update(image_dto) => {
                info!("Update image_dto: {}", image_dto.id);
                info!("Update image_dto: {:?}", image_dto.tags);
                Action::NavigateToUpdate(image_dto)
            }

            Message::OpenLocalImage(id) => {
                let img = self.images.iter().find(|img| img.id == id).unwrap();

                let path_buf = if !img.image_dto.is_folder {
                    Path::new(&img.image_dto.path)
                        .parent()
                        .expect("Image path should have a parent")
                        .to_path_buf()
                } else {
                    Path::new(&img.image_dto.path).to_path_buf()
                };

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
                        // Usar a nova função de deleção inteligente
                        // from_folder = false (imagem principal/pasta)
                        if let Err(e) = file_service::delete_image_smart(&dto.path, false).await {
                            error!("Failed to delete image files: {}", e);
                        }

                        // Deletar do banco de dados
                        if let Err(e) = image_service::delete_image(dto.id).await {
                            error!("Failed to delete image from database: {}", e);
                        }
                    },
                    |_| {
                        push_success(t!("message.delete.success"));
                        Message::NoOps
                    },
                );
                Action::Run(task)
            }

            Message::DeleteImageFromFolder(dto) => {
                self.images.retain(|img| img.id != dto.id);
                let task = Task::perform(
                    async move {
                        // Usar a nova função de deleção inteligente
                        if let Err(e) = file_service::delete_image_smart(&dto.path, true).await {
                            error!("Failed to delete image file from folder: {}", e);
                        }
                    },
                    |_| {
                        push_success(t!("message.delete.success"));
                        Message::NoOps
                    },
                );
                Action::Run(task)
            }

            Message::PushContainer(images, current_page, total_pages, is_from_folder) => {
                self.images.reserve(images.len());

                info!("Pushing {} images", images.len());
                for img in images {
                    info!("Pushing image {}", img.id);
                    info!(
                        "Tags: {:?}",
                        img.tags.iter().map(|t| &t.name).collect::<Vec<_>>()
                    );
                    self.images
                        .push(ImageContainer::new(img.clone(), is_from_folder));
                }

                set_current_page(current_page);
                self.current_page = current_page;
                self.total_pages = total_pages;

                Action::Run(self.change_scroll())
            }

            Message::OpenImage(image_dto) => {
                if image_dto.is_folder {
                    info!("Opening folder {}", image_dto.path);
                    self.images.clear();
                    self.folder_opened = true;
                    self.show_preview = false;
                    let task = Task::perform(
                        async move {
                            let sub_images = file_service::expand_folder_dto(&image_dto);
                            sub_images
                        },
                        |sub_images| Message::PushContainer(sub_images, 0, 0, true),
                    );
                    Action::Run(task)
                } else {
                    // Find the index of the image being opened
                    if let Some(index) = self
                        .images
                        .iter()
                        .position(|img| img.image_dto.id == image_dto.id)
                    {
                        self.current_preview_index = index;
                        self.show_preview = true;

                        if image_dto.is_folder {
                            self.preview_handle =
                                Handle::from_path(image_dto.thumbnail_path.clone());
                        } else {
                            self.preview_handle = Handle::from_path(image_dto.path.clone());
                        }
                    }
                    Action::None
                }
            }

            Message::PreviousImage => {
                self.change_preview(-1);
                Action::None
            }

            Message::NextImage => {
                self.change_preview(1);
                Action::None
            }

            Message::ClosePreview => {
                self.show_preview = false;
                self.preview_handle = Handle::from_path("".to_string());
                self.current_preview_index = 0;

                Action::Run(self.change_scroll())
            }

            Message::CloseFolder => {
                self.images.clear();
                self.folder_opened = false;
                let task = Task::perform(async {}, |_| Message::SearchButtonPressed);
                Action::Run(task)
            }

            Message::TagsLoaded(tags) => {
                self.tag_selector.available = tags;
                Action::None
            }

            Message::TagSelectorMessage(msg) => {
                // Update the tag selector state with the incoming message
                let _ = self.tag_selector.update(msg);

                // Get the currently selected tags and save them globally
                let selected_tags = self.tag_selector.selected.clone();
                set_selected_tags(selected_tags.clone());

                // Debug log to verify tags are being saved globally
                info!(
                    "Saved tags to global: {:?}",
                    selected_tags.iter().map(|t| &t.name).collect::<Vec<_>>()
                );

                // Trigger a search task asynchronously
                let task = Task::perform(async move {}, |_| Message::SearchButtonPressed);
                Action::Run(task)
            }

            Message::GoToPage(page_index) => {
                let page_size = self.page_size;
                self.images.clear();
                let query = self.query.clone();
                let selected_tags = self.tag_selector.selected.clone();
                self.scroll_offset = 0.0;
                set_scroll_offset(0.0);
                let task = Task::perform(
                    async move {
                        let mut filter = Filter::new();

                        if !query.is_empty() {
                            filter.query = query;
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
                        Message::PushContainer(images, current_page, total_pages, false)
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
                        Message::PushContainer(images, current_page, total_pages, false)
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

    pub fn view(&'_ self) -> Element<'_, Message> {
        // Close folder header
        let close_folder: Element<Message> = if self.folder_opened {
            header::header(|| Message::CloseFolder)
        } else {
            Container::new(Space::new(Length::Shrink, Length::Shrink))
                .width(Length::Fill)
                .into()
        };

        // Tags view
        let tags_view = Container::new(
            self.tag_selector
                .view()
                .map(Message::TagSelectorMessage),
        )
            .width(Length::Fill)
            .padding(10)
            .style(Modern::card_container());

        let search_bar = search_bar::search_bar(search_bar::SearchBarConfig {
            query: &self.query,
            sort_order: self.selected_sort_order.clone(),
            sort_options: &[SortOrder::CreatedAsc, SortOrder::CreatedDesc],
            on_query_change: Box::new(Message::QueryChanged),
            on_search: Message::SearchButtonPressed,
            on_register: Message::NavigateToRegister,
            on_sort_change: Box::new(Message::SortOrderChanged),
        });

        // Header
        let header = Column::new().spacing(20).push(search_bar).push(tags_view);

        // Image grid
        let mut images_row = Row::new().spacing(20);
        for image in &self.images {
            images_row = images_row.push(image.view());
        }

        let images_grid = if self.images.is_empty() {
            empty_state::empty_state(
                "image",
                "No images found",
                "Try adjusting your search criteria",
            )
        } else {
            Container::new(
                Column::new()
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .push(close_folder)
                    .push(
                        Scrollable::new(
                            Container::new(images_row.wrap())
                                .width(Length::Fill)
                                .align_x(Horizontal::Center)
                                .padding(20),
                        )
                            .id(self.scroll_id.clone())
                            .on_scroll(Message::ScrollChanged)
                            .width(Length::Fill)
                            .height(Length::Fill),
                    ),
            )
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        let images_container = Container::new(images_grid)
            .style(Modern::card_container())
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20);

        let pagination_view = pagination::pagination(
            self.current_page,
            self.total_pages,
            Message::GoToPage,
        );

        let content = Column::new()
            .spacing(30)
            .push(header)
            .push(images_container)
            .push(pagination_view);

        let layout = Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20);

        // Image preview
        if self.show_preview {
            let preview_config = image_preview_modal::PreviewConfig {
                handle: self.preview_handle.clone(),
                current_index: self.current_preview_index,
                total_images: self.images.len(),
                on_close: Message::ClosePreview,
                on_previous: if self.images.len() > 1 {
                    Some(Message::PreviousImage)
                } else {
                    None
                },
                on_next: if self.images.len() > 1 {
                    Some(Message::NextImage)
                } else {
                    None
                },
            };
            image_preview_modal::image_preview_modal(preview_config)
        } else {
            layout.into()
        }
    }
}
