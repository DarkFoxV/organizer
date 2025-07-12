use iced::alignment::Horizontal;
use iced::widget::{Column, button, container, text, scrollable};
use iced::{Element, Length, Task};
use iced_modern_theme::Modern;
use log::info;
use rust_i18n::t;
use crate::config::Settings;

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
            },
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
                .height(Length::Fixed(45.0))
                .padding(10)
                .on_press(Message::ButtonSignal(id));

            if id == selected {
                base.style(Modern::green_tinted_button())
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
            .spacing(5);

        let empty_middle = scrollable(Column::new().push(text("").size(1)))
            .width(Length::Fill)
            .height(Length::Fill);

        let settings_button = Column::new().push(
            button(
                text(t!("navbar.button.settings"))
                    .width(Length::Fill)
                    .align_x(Horizontal::Center),
            )
                .width(Length::Fill)
                .height(Length::Fixed(45.0))
                .padding(10)
                .on_press(Message::ButtonSignal(NavButton::Preferences))
                .style(Modern::blue_tinted_button()),
        );

        let layout = Column::new()
            .push(navbar.height(Length::Fixed(195.0)))
            .push(empty_middle.height(Length::Fill))
            .push(settings_button.height(Length::Fixed(45.0)))
            .spacing(10);

        container(layout)
            .width(Length::Fixed(250.0))
            .height(Length::Fill)
            .padding(5)
            .style(Modern::sidebar_container())
            .into()
    }
}
