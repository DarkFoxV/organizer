use iced::widget::{Column, Container, PickList, Text, TextInput};
use iced::{Element, Length, Task};
use iced_modern_theme::Modern;
use log::error;
use crate::config::{get_settings, get_settings_mut};

pub enum Action {
    None,
    UpdateUI(),
}

#[derive(Debug, Clone)]
pub enum Message {
    LanguageChanged(String),
    ThemeChanged(String),
    ItemsPerPageChanged(u64),
    NoOps,
}

pub struct Preferences {
    available_languages: Vec<String>,
    pub theme: String,
    pub items_per_page: u64,
    selected_language: String,
}

const THEMES: [&str; 3] = ["Light", "Dark", "System"];

impl Preferences {
    pub fn new() -> (Self, Task<Message>) {
        let settings = get_settings();
        let selected_language = settings.config.language.clone();
        let theme = settings.config.theme.clone();
        let items_per_page = settings.config.items_per_page;
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
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::LanguageChanged(language) => {
                let mut settings = get_settings_mut();
                settings.config.language = language; // move
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
                settings.config.items_per_page = self.items_per_page.clone();
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

        let language_picker = PickList::new(
            language_options,
            Some(self.selected_language.clone()),
            Message::LanguageChanged,
        )
            .placeholder(t!("preferences.select.language"))
            .style(Modern::pick_list());


        let theme_picker = PickList::new(THEMES, Some(self.theme.as_str()), |theme| {
            Message::ThemeChanged(theme.to_string())
        })
        .placeholder(t!("preferences.select.theme"))
        .style(Modern::pick_list());

        // Campo numérico para items per page
        let items_input = number_input(
            self.items_per_page,
            100, // Valor máximo
            Message::ItemsPerPageChanged,
        )
        .style(Modern::text_input());

        let content = Column::new()
            .padding(20)
            .spacing(20)
            .push(
                Text::new(t!("preferences.title"))
                    .size(24)
                    .style(Modern::primary_text()),
            )
            .push(
                Column::new()
                    .spacing(10)
                    .push(Text::new(t!("preferences.label.language")))
                    .push(language_picker)
                    .push(
                        Column::new()
                            .spacing(10)
                            .push(Text::new(t!("preferences.label.theme")))
                            .push(theme_picker),
                    )
                    .push(
                        Column::new()
                            .spacing(10)
                            .push(Text::new(t!("preferences.label.items_per_page")))
                            .push(items_input),
                    ),
            );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn number_input<'a>(
    value: u64,
    max: u64,
    on_change: impl Fn(u64) -> Message + 'a,
) -> TextInput<'a, Message> {
    TextInput::new("", &value.to_string()).on_input(move |s| {
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
}
