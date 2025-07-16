use crate::config::Settings;
use iced::alignment::Horizontal;
use iced::widget::{Column, button, container, scrollable, text};
use iced::{Element, Length, Padding, Task};
use iced_modern_theme::Modern;
use log::info;
use rust_i18n::t;

pub enum Action {
    Run(Task<Message>),
    Navigate(NavButton),
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavButton {
    Home,
    Search,
    Workspace,
    ManageTags,
    Preferences,
}

#[derive(Debug, Clone)]
pub enum Message {
    ButtonSignal(NavButton),
    ButtonPressed(NavButton),
    NoOps,
}

pub struct Navbar {
    pub selected: NavButton,
    settings: Settings,
}

impl Navbar {
    pub fn new() -> Self {
        let settings = Settings::load();
        Navbar {
            selected: NavButton::Search,
            settings,
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ButtonSignal(id) => {
                self.selected = id;
                Action::Run(Task::perform(async {}, move |_| Message::ButtonPressed(id)))
            }
            Message::ButtonPressed(id) => {
                self.selected = id;
                Action::Navigate(id)
            }
            Message::NoOps => {
                self.settings = Settings::load();
                info!("navbar update ");
                Action::None
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        fn styled_button(
            label: String,
            id: NavButton,
            selected: NavButton,
        ) -> iced::widget::Button<'static, Message> {
            let base = button(text(label).width(Length::Fill).align_x(Horizontal::Center))
                .width(Length::Fill)
                .height(Length::Fixed(48.0))
                .padding(Padding {
                    top: 12.0,
                    right: 16.0,
                    bottom: 12.0,
                    left: 16.0,
                })
                .on_press(Message::ButtonSignal(id));

            if id == selected {
                base.style(Modern::primary_button())
            } else {
                base.style(Modern::blue_tinted_button())
            }
        }

        let navbar = Column::new()
            .push(styled_button(
                t!("navbar.button.home").to_string(),
                NavButton::Home,
                self.selected,
            ))
            .push(styled_button(
                t!("navbar.button.search").to_string(),
                NavButton::Search,
                self.selected,
            ))
            .push(styled_button(
                t!("navbar.button.workspace").to_string(),
                NavButton::Workspace,
                self.selected,
            ))
            .spacing(5)
            .push(styled_button(
                t!("navbar.button.manage_tags").to_string(),
                NavButton::ManageTags,
                self.selected,
            ))
            .spacing(5);

        let empty_middle = scrollable(Column::new().push(text("").size(1)))
            .width(Length::Fill)
            .height(Length::Fill);

        let settings_button = Column::new().push(
            styled_button(
                t!("navbar.button.settings").to_string(),
                NavButton::Preferences,
                self.selected,
            )
            .padding(Padding {
                top: 12.0,
                right: 16.0,
                bottom: 12.0,
                left: 16.0,
            })
            .on_press(Message::ButtonSignal(NavButton::Preferences)),
        );

        let layout = Column::new()
            .push(navbar.height(Length::Fixed(225.0)))
            .push(empty_middle.height(Length::Fill))
            .push(settings_button.height(Length::Fixed(48.0)))
            .spacing(10);

        container(layout)
            .width(Length::Fixed(280.0))
            .height(Length::Fill)
            .padding(5)
            .style(Modern::sidebar_container())
            .into()
    }
}
