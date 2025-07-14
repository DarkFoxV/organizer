#[macro_use]
extern crate rust_i18n;
mod components;
mod config;
mod dtos;
mod models;
mod screen;
mod services;

use crate::components::navbar::{NavButton, Navbar};
use crate::components::toast_view::ToastView;
use crate::components::{navbar, toast_view};
use crate::config::get_settings;
use crate::dtos::image_dto::ImageDTO;
use crate::models::toast::Toast;
use crate::screen::update::Update;
use crate::screen::{Preferences, preferences, search};
use crate::screen::{Register, Screen, Search};
use crate::screen::{register, update};
use crate::services::{clipboard_service, database_service, logger_service, toast_service};
use iced::event;
use iced::keyboard;
use iced::widget::{Column, Row, container, stack};
use iced::{Alignment, Element, Event, Length, Subscription, Task, Theme, time};
use iced_modern_theme::Modern;
use image::DynamicImage;
use log::info;
use std::time::{Duration, Instant};

i18n!("locales", fallback = "en");

#[derive(Debug, Clone)]
pub enum Message {
    Navbar(navbar::Message),
    Search(search::Message),
    Register(register::Message),
    Update(update::Message),
    Preferences(preferences::Message),
    SettingsUpdated,
    Toast(toast_view::Message),
    Tick(Instant),
    HandleToast(Toast),
    EscapePressed,
    PasteShortcut,
    Navigate(NavigationTarget),
    NoOps,
}

#[derive(Debug, Clone)]
pub enum NavigationTarget {
    Search,
    Register(Option<DynamicImage>),
    Update(ImageDTO),
    Preferences,
}

pub struct Organizer {
    theme: Theme,
    screen: Screen,
    navbar: Navbar,
    toasts: Vec<ToastView>,
}

impl Organizer {
    pub fn new() -> (Self, Task<Message>) {
        let (search, search_task) = Search::new();
        let task = search_task.map(Message::Search);
        let settings = get_settings();
        let theme = Self::get_theme_from_settings(&settings);

        (
            Self {
                theme,
                screen: Screen::Search(search),
                navbar: Navbar::new(),
                toasts: vec![],
            },
            task,
        )
    }

    pub fn title(&self) -> String {
        t!("app.title").to_string()
    }

    pub fn toast(&mut self, toast: ToastView) {
        self.toasts.push(toast);
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn get_theme_from_settings(settings: &config::Settings) -> Theme {
        match settings.config.theme.as_str() {
            "Dark" => Modern::dark_theme(),
            "Light" => Modern::light_theme(),
            _ => Default::default(),
        }
    }

    // Method to navigate to different screens
    fn navigate_to(&mut self, target: NavigationTarget) -> Task<Message> {
        match target {
            NavigationTarget::Search => {
                let (search, task) = Search::new();
                self.screen = Screen::Search(search);
                self.navbar.selected = NavButton::Search;
                task.map(Message::Search)
            }
            NavigationTarget::Register(image) => {
                let (register, task) = Register::new(image);
                self.screen = Screen::Register(register);
                task.map(Message::Register)
            }
            NavigationTarget::Update(dto) => {
                let (update, task) = Update::new(dto);
                self.screen = Screen::Update(update);
                task.map(Message::Update)
            }
            NavigationTarget::Preferences => {
                let (preferences, task) = Preferences::new();
                self.screen = Screen::Preferences(preferences);
                self.navbar.selected = NavButton::Preferences;
                task.map(Message::Preferences)
            }
        }
    }

    // Method to handle escape key
    fn handle_escape(&mut self) -> Task<Message> {
        match &mut self.screen {
            Screen::Search(_) => {
                let msg = Message::Search(search::Message::ClosePreview);
                Task::perform(async move { msg }, |m| m)
            }
            _ => self.navigate_to(NavigationTarget::Search),
        }
    }

    // Method to handle paste shortcut
    fn handle_paste(&mut self) -> Task<Message> {
        let dynamic_image = clipboard_service::get_clipboard_image();

        if let Some(image) = dynamic_image {
            match &mut self.screen {
                Screen::Search(search) => {
                    info!("Pasting image to search");
                    let action = search.update(search::Message::ImagePasted(image));
                    match action {
                        search::Action::NavigatorToRegister(image) => {
                            self.navigate_to(NavigationTarget::Register(image))
                        }
                        _ => Task::none(),
                    }
                }
                Screen::Register(register) => {
                    info!("Pasting image to register");
                    register.update(register::Message::ImagePasted(image));
                    Task::none()
                }
                _ => Task::none(),
            }
        } else {
            Task::none()
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(target) => self.navigate_to(target),

            Message::HandleToast(mut toast) => {
                toast.duration = Duration::from_secs(4);
                self.toasts.push(ToastView { toast });
                Task::none()
            }

            Message::SettingsUpdated => {
                let settings = get_settings();
                self.theme = Self::get_theme_from_settings(&settings);
                self.navbar.update(navbar::Message::NoOps);
                self.navigate_to(NavigationTarget::Preferences)
            }

            Message::EscapePressed => self.handle_escape(),

            Message::PasteShortcut => self.handle_paste(),

            Message::Search(message) => {
                if let Screen::Search(search) = &mut self.screen {
                    let action = search.update(message);

                    match action {
                        search::Action::None => Task::none(),
                        search::Action::Run(task) => task.map(Message::Search),
                        search::Action::NavigateToUpdate(dto) => {
                            self.navigate_to(NavigationTarget::Update(dto))
                        }
                        search::Action::NavigatorToRegister(dynamic_image) => {
                            self.navigate_to(NavigationTarget::Register(dynamic_image))
                        }
                    }
                } else {
                    Task::none()
                }
            }

            Message::Register(message) => {
                if let Screen::Register(register) = &mut self.screen {
                    let action = register.update(message);

                    match action {
                        register::Action::None => Task::none(),
                        register::Action::Run(task) => task.map(Message::Register),
                        register::Action::GoToSearch => self.navigate_to(NavigationTarget::Search),
                    }
                } else {
                    Task::none()
                }
            }

            Message::Update(message) => {
                if let Screen::Update(update) = &mut self.screen {
                    let action = update.update(message);

                    match action {
                        update::Action::None => Task::none(),
                        update::Action::Run(task) => task.map(Message::Update),
                        update::Action::GoToSearch => self.navigate_to(NavigationTarget::Search),
                    }
                } else {
                    Task::none()
                }
            }

            Message::Preferences(message) => {
                if let Screen::Preferences(preferences) = &mut self.screen {
                    let action = preferences.update(message);

                    match action {
                        preferences::Action::None => Task::none(),
                        preferences::Action::UpdateUI() => {
                            Task::perform(async { Message::SettingsUpdated }, |m| m)
                        }
                    }
                } else {
                    Task::none()
                }
            }

            Message::Navbar(navbar_msg) => {
                info!("Navbar message: {:?}", navbar_msg);
                let action = self.navbar.update(navbar_msg);

                match action {
                    navbar::Action::Run(task) => task.map(Message::Navbar),
                    navbar::Action::Navigate(button) => {
                        let target = match button {
                            NavButton::Home | NavButton::Search => NavigationTarget::Search,
                            NavButton::Workspace => NavigationTarget::Register(None),
                            NavButton::Preferences => NavigationTarget::Preferences,
                        };
                        self.navigate_to(target)
                    }
                    navbar::Action::None => Task::none(),
                }
            }

            Message::Tick(now) => {
                self.toasts.retain(|toast| {
                    now.duration_since(toast.toast.created) < Duration::from_secs(4)
                });
                Task::none()
            }

            Message::Toast(toast_view::Message::Dismiss(id)) => {
                self.toasts.retain(|toast| toast.toast.id != Some(id));
                Task::none()
            }

            Message::NoOps => Task::none(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![time::every(Duration::from_millis(100)).map(|_| {
            if let Some(toast) = toast_service::pop_toast() {
                info!("Popping toast: {}", toast.message);
                Message::HandleToast(toast)
            } else {
                Message::Tick(Instant::now())
            }
        })];

        let keyboard_subscription = match &self.screen {
            Screen::Register(_) | Screen::Update(_) | Screen::Search(_) => {
                event::listen().map(|event| match event {
                    Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => match key {
                        keyboard::Key::Named(keyboard::key::Named::Escape) => {
                            Message::EscapePressed
                        }
                        _ => Message::NoOps,
                    },
                    _ => Message::NoOps,
                })
            }
            _ => Subscription::none(),
        };

        let clipboard_subscription = match &self.screen {
            Screen::Register(_) | Screen::Search(_) => event::listen().map(|event| match event {
                Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => match key {
                    keyboard::Key::Character(ref c) if c == "v" && modifiers.control() => {
                        Message::PasteShortcut
                    }
                    _ => Message::NoOps,
                },
                _ => Message::NoOps,
            }),
            _ => Subscription::none(),
        };

        subscriptions.push(clipboard_subscription);
        subscriptions.push(keyboard_subscription);
        Subscription::batch(subscriptions)
    }

    pub fn view(&self) -> Element<Message> {
        let navbar = self.navbar.view().map(Message::Navbar);

        let content = match &self.screen {
            Screen::Search(search) => search.view().map(Message::Search),
            Screen::Register(register) => register.view().map(Message::Register),
            Screen::Update(update) => update.view().map(Message::Update),
            Screen::Preferences(preferences) => preferences.view().map(Message::Preferences),
        };

        let layout = Row::new().push(navbar).push(content);

        let toast_widgets: Vec<_> = self
            .toasts
            .iter()
            .map(|toast| toast.view().map(Message::Toast))
            .collect();

        let toast_overlay = container(Column::with_children(toast_widgets).spacing(10))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .align_x(Alignment::Start)
            .align_y(Alignment::End);

        stack![layout, toast_overlay].into()
    }
}

fn main() -> iced::Result {
    info!("Starting application");
    logger_service::init().expect("Failed to initialize logger");

    info!("{:?}", _rust_i18n_available_locales());

    {
        let settings = get_settings();
        rust_i18n::set_locale(settings.config.language.as_str());
    }

    // Create Tokio runtime
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    // Start database
    rt.block_on(async {
        dotenv::dotenv().ok();
        database_service::prepare_database().await.unwrap();
    });

    rt.shutdown_background();

    // Start application
    iced::application(Organizer::title, Organizer::update, Organizer::view)
        .theme(Organizer::theme)
        .subscription(Organizer::subscription)
        .run_with(Organizer::new)
}
