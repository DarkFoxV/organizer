use crate::config::{get_settings, get_settings_mut};
use iced::widget::{Column, Container, PickList, Row, Scrollable, Slider, Text, TextInput};
use iced::{Element, Length, Padding, Task};
use iced_modern_theme::Modern;
use log::error;

pub enum Action {
    None,
    UpdateUI(),
}

#[derive(Debug, Clone)]
pub enum Message {
    LanguageChanged(String),
    ThemeChanged(String),
    ItemsPerPageChanged(u64),
    ThumbCompressionChanged(u8),
    ImageCompressionChanged(u8),
    NoOps,
}

pub struct Preferences {
    available_languages: Vec<String>,
    pub theme: String,
    pub items_per_page: u64,
    pub thumb_compression: u8,
    pub image_compression: u8,
    selected_language: String,
}

const THEMES: [&str; 3] = ["Light", "Dark", "System"];

impl Preferences {
    pub fn new() -> (Self, Task<Message>) {
        let settings = get_settings();
        let selected_language = settings.config.language.clone();
        let theme = settings.config.theme.clone();
        let items_per_page = settings.config.items_per_page;
        let thumb_compression = settings.config.thumb_compression.unwrap_or(9);
        let image_compression = settings.config.image_compression.unwrap_or(5);
        let available_languages = rust_i18n::available_locales!()
            .iter()
            .map(|l| l.to_string())
            .collect();
        (
            Self {
                available_languages,
                selected_language,
                theme,
                items_per_page,
                thumb_compression,
                image_compression,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::LanguageChanged(language) => {
                let mut settings = get_settings_mut();
                settings.config.language = language;
                if let Err(err) = settings.save() {
                    eprintln!("Failed to save settings: {}", err);
                }
                rust_i18n::set_locale(&settings.config.language);
                self.selected_language = settings.config.language.clone();
                Action::UpdateUI()
            }
            Message::ThemeChanged(theme) => {
                let mut settings = get_settings_mut();
                settings.config.theme = theme;
                if let Err(err) = settings.save() {
                    error!("Failed to save settings: {}", err);
                }
                self.theme = settings.config.theme.clone();
                Action::UpdateUI()
            }
            Message::ItemsPerPageChanged(items_per_page) => {
                self.items_per_page = items_per_page.clamp(1, 100);
                let mut settings = get_settings_mut();
                settings.config.items_per_page = self.items_per_page;
                if let Err(err) = settings.save() {
                    error!("Failed to save settings: {}", err);
                }
                Action::None
            }
            Message::ThumbCompressionChanged(compression) => {
                self.thumb_compression = compression.clamp(0, 9);
                let mut settings = get_settings_mut();
                settings.config.thumb_compression = Some(self.thumb_compression);
                if let Err(err) = settings.save() {
                    error!("Failed to save settings: {}", err);
                }
                Action::None
            }
            Message::ImageCompressionChanged(compression) => {
                self.image_compression = compression.clamp(0, 9);
                let mut settings = get_settings_mut();
                settings.config.image_compression = Some(self.image_compression);
                if let Err(err) = settings.save() {
                    error!("Failed to save settings: {}", err);
                }
                Action::None
            }
            Message::NoOps => Action::None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let language_options = self.available_languages.clone();

        // Language Section
        let language_section = self.create_section(
            t!("preferences.label.language").to_string(),
            PickList::new(
                language_options,
                Some(self.selected_language.clone()),
                Message::LanguageChanged,
            )
            .placeholder(t!("preferences.select.language"))
            .style(Modern::pick_list())
            .width(Length::Fill),
        );

        // Theme Section
        let theme_section = self.create_section(
            t!("preferences.label.theme").to_string(),
            PickList::new(THEMES, Some(self.theme.as_str()), |theme| {
                Message::ThemeChanged(theme.to_string())
            })
            .placeholder(t!("preferences.select.theme"))
            .style(Modern::pick_list())
            .width(Length::Fill),
        );

        // Items per Page Section
        let items_section = self.create_section(
            t!("preferences.label.items_per_page").to_string(),
            number_input(self.items_per_page, 100, Message::ItemsPerPageChanged)
                .style(Modern::text_input())
                .width(Length::Fill),
        );

        // Thumb Compression Section
        let thumb_compression_section = self.create_compression_section(
            t!("preferences.label.thumb_compression").to_string(),
            self.thumb_compression,
            Message::ThumbCompressionChanged,
        );

        // Image Compression Section
        let image_compression_section = self.create_compression_section(
            t!("preferences.label.image_compression").to_string(),
            self.image_compression,
            Message::ImageCompressionChanged,
        );

        let scrollable = Scrollable::new(
            Column::new()
                .padding(20)
                .spacing(30)
                .push(
                    Text::new(t!("preferences.title"))
                        .size(32)
                        .style(Modern::primary_text()),
                )
                .push(
                    Text::new(t!("preferences.subtitle"))
                        .size(16)
                        .style(Modern::secondary_text()),
                )
                .push(
                    Column::new()
                        .spacing(25)
                        .push(language_section)
                        .push(theme_section)
                        .push(items_section)
                        .push(thumb_compression_section)
                        .push(image_compression_section),
                ),
        );

        Container::new(scrollable)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn create_section<'a>(
        &self,
        title: String,
        widget: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        Container::new(
            Column::new()
                .spacing(12)
                .push(Text::new(title).size(18).style(Modern::primary_text()))
                .push(widget),
        )
        .padding(20)
        .style(Modern::card_container())
        .width(Length::Fill)
        .into()
    }

    fn create_compression_section<'a>(
        &self,
        title: String,
        value: u8,
        on_change: fn(u8) -> Message,
    ) -> Element<'a, Message> {
        let slider = Slider::new(0..=9, value, on_change).width(Length::Fill);

        let value_display = Container::new(
            Text::new(format!("{}", value))
                .size(16)
                .style(Modern::primary_text()),
        )
        .padding(Padding::new(8.0))
        .style(Modern::card_container());

        let quality_text = Text::new(match value {
            0..=2 => t!("preferences.compression.low").to_string(),
            3..=5 => t!("preferences.compression.medium").to_string(),
            6..=7 => t!("preferences.compression.high").to_string(),
            8..=9 => t!("preferences.compression.max").to_string(),
            _ => "None".to_string(),
        })
        .size(14)
        .style(Modern::secondary_text());

        Container::new(
            Column::new()
                .spacing(12)
                .push(
                    Row::new()
                        .spacing(10)
                        .push(Text::new(title).size(18).style(Modern::primary_text()))
                        .push(value_display),
                )
                .push(
                    Row::new()
                        .spacing(15)
                        .push(Text::new("0").size(12).style(Modern::secondary_text()))
                        .push(slider)
                        .push(Text::new("9").size(12).style(Modern::secondary_text())),
                )
                .push(quality_text),
        )
        .padding(20)
        .style(Modern::card_container())
        .width(Length::Fill)
        .into()
    }
}

fn number_input<'a>(
    value: u64,
    max: u64,
    on_change: impl Fn(u64) -> Message + 'a,
) -> TextInput<'a, Message> {
    TextInput::new("", &value.to_string())
        .on_input(move |s| {
            if let Ok(num) = s.parse::<u64>() {
                if num <= max {
                    on_change(num)
                } else {
                    on_change(max)
                }
            } else if s.is_empty() {
                on_change(1)
            } else {
                on_change(value)
            }
        })
        .padding(Padding::new(12.0))
        .size(16)
}
