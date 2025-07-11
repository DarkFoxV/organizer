#[macro_use]
extern crate rust_i18n;
mod components;
mod config;
mod models;
mod screen;
mod services;

use crate::components::navbar::{NavButton, Navbar};
use crate::components::toast::{Toast, ToastKind};
use crate::components::{navbar, toast};
use crate::screen::update::Update;
use crate::screen::{Preferences, preferences, search};
use crate::screen::{Register, Screen, Search};
use crate::screen::{register, update};
use crate::services::{database_service, logger_service};
use iced::widget::{Column, Row, container, stack};
use iced::{Alignment, Element, Length, Subscription, Task, Theme, time};
use std::time::{Duration, Instant};
use iced_modern_theme::Modern;
use log::info;
use crate::config::get_settings;

i18n!("locales", fallback = "en");

#[derive(Debug, Clone)]
pub enum Message {
    Navbar(navbar::Message),
    Search(search::Message),
    Register(register::Message),
    Update(update::Message),
    Preferences(preferences::Message),
    SettingsUpdated,
    Toast(toast::Message),
    Tick(Instant),
    HandleToast{
        kind: ToastKind,
        message: String,
        duration: Option<Duration>,
    },
}


pub struct Organizer {
    theme: Theme,
    screen: Screen,
    navbar: Navbar,
    toasts: Vec<Toast>,
    next_toast_id: u32,
}


impl Organizer {
    pub fn new() -> (Self, Task<Message>) {
        let (search, search_task) = Search::new();
        let task = search_task.map(Message::Search);
        let settings = get_settings();
        let theme = if settings.config.theme == "Dark" { Modern::dark_theme() } else { Modern::light_theme() };
        (
            Self {
                theme,
                screen: Screen::Search(search),
                navbar: Navbar::new(),
                toasts: vec![],
                next_toast_id: 0,
            },
            task,
        )
    }

    pub fn title(&self) -> String {
        t!("app.title").to_string()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::HandleToast {
                kind,
                message,
                duration,
            } => {
                self.toasts.push(Toast {
                    id: self.next_toast_id,
                    message,
                    kind,
                    created: Instant::now(),
                    duration: duration.unwrap_or(Duration::from_secs(4)),
                });
                self.next_toast_id += 1;
                Task::none()
            }
            Message::Register(register::Message::ShowToast {
                kind,
                message,
                duration,
            }) => {
                self.toasts.push(Toast {
                    id: self.next_toast_id,
                    message,
                    kind,
                    created: Instant::now(),
                    duration: duration.unwrap_or(Duration::from_secs(4)),
                });
                self.next_toast_id += 1;
                Task::none()
            }
            Message::Update(update::Message::ShowToast {
                kind,
                message,
                duration,
            }) => {
                self.toasts.push(Toast {
                    id: self.next_toast_id,
                    message,
                    kind,
                    created: Instant::now(),
                    duration: duration.unwrap_or(Duration::from_secs(4)),
                });
                self.next_toast_id += 1;
                Task::none()
            }
            Message::Search(message) => {
                if let Screen::Search(search) = &mut self.screen {
                    let action = search.update(message);

                    match action {
                        search::Action::None => Task::none(),
                        search::Action::Run(task) => task.map(Message::Search),
                        search::Action::ShowToast {kind, message, duration} => {
                            self.update(Message::HandleToast {
                                kind,
                                message,
                                duration
                            })
                        },
                        search::Action::NavigateToUpdate(dto) => {
                            let (update, task) = Update::new(dto);
                            self.screen = Screen::Update(update);
                            task.map(Message::Update)
                        },

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
                            let _ = self.update(Message::SettingsUpdated);
                            Task::none()
                        }
                    }
                } else {
                    Task::none()
                }
            }

            Message::SettingsUpdated => {
                let settings = get_settings();
                self.theme = if settings.config.theme == "Dark" {
                    Modern::dark_theme()
                } else {
                    Modern::light_theme()
                };
                self.navbar.update(navbar::Message::NoOps);
                let (preferences, _task) = Preferences::new();
                self.screen = Screen::Preferences(preferences);

                Task::none()
            }
            Message::Update(message) => {
                if let Screen::Update(update) = &mut self.screen {
                    let action = update.update(message);

                    fn to_task(app: &mut Organizer, action: update::Action) -> Task<Message> {
                        match action {
                            update::Action::None => Task::none(),
                            update::Action::Run(task) => task.map(Message::Update),
                            update::Action::Batch(batch) => {
                                let tasks: Vec<Task<Message>> =
                                    batch.into_iter().map(|a| to_task(app, a)).collect();
                                Task::batch(tasks)
                            }
                            update::Action::GoToSearch => {
                                let (search, task) = Search::new();
                                app.screen = Screen::Search(search);
                                task.map(Message::Search)
                            }
                        }
                    }
                    to_task(self, action).into()
                } else {
                    Task::none()
                }
            }
            Message::Register(message) => {
                if let Screen::Register(register) = &mut self.screen {
                    let action = register.update(message);

                    // Função recursiva inline
                    fn to_task(app: &mut Organizer, action: register::Action) -> Task<Message> {
                        match action {
                            register::Action::None => Task::none(),
                            register::Action::Run(task) => task.map(Message::Register),
                            register::Action::Batch(batch) => {
                                let tasks: Vec<Task<Message>> =
                                    batch.into_iter().map(|a| to_task(app, a)).collect();
                                Task::batch(tasks)
                            }
                            register::Action::GoToSearch => {
                                let (search, task) = Search::new();
                                app.screen = Screen::Search(search);
                                task.map(Message::Search)
                            }
                        }
                    }
                    to_task(self, action).into()
                } else {
                    Task::none()
                }
            }
            Message::Navbar(navbar_msg) => {
                let action = self.navbar.update(navbar_msg);

                match action {
                    navbar::Action::Run(task) => task.map(Message::Navbar),
                    navbar::Action::Navigate(id) => match id {
                        NavButton::Home => {
                            let (search, task) = Search::new();
                            self.screen = Screen::Search(search);
                            task.map(Message::Search)
                        }
                        NavButton::Search => {
                            let (search, task) = Search::new();
                            self.screen = Screen::Search(search);
                            task.map(Message::Search)
                        }
                        NavButton::Register => {
                            let (register, task) = Register::new();
                            self.screen = Screen::Register(register);
                            task.map(Message::Register)
                        }
                        NavButton::Workspace => {
                            let (register, task) = Register::new();
                            self.screen = Screen::Register(register);
                            task.map(Message::Register)
                        }
                        NavButton::Preferences => {
                            let (preferences, task) = Preferences::new();
                            self.screen = Screen::Preferences(preferences);
                            task.map(Message::Preferences)
                        }
                    },
                    navbar::Action::None => Task::none(),
                }
            }
            Message::Tick(now) => {
                self.toasts
                    .retain(|toast| now.duration_since(toast.created) < Duration::from_secs(4));
                Task::none()
            }
            Message::Toast(toast::Message::Dismiss(id)) => {
                self.toasts.retain(|toast| toast.id != id);
                Task::none()
            },
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(500)).map(Message::Tick)
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

    // Cria runtime Tokio
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    // Roda inicializações assíncronas
    rt.block_on(async {
        dotenv::dotenv().ok();
        database_service::prepare_database().await.unwrap();
    });

    rt.shutdown_background();

    // Inicia o app Iced
    iced::application(Organizer::title, Organizer::update, Organizer::view)
        .theme(Organizer::theme)
        .subscription(Organizer::subscription)
        .run_with(Organizer::new)
}
