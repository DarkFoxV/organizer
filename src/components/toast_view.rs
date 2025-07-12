use crate::models::toast::Toast;
use iced::alignment::Vertical;
use iced::widget::{Column, Container, Row, Space, Text, button};
use iced::{Alignment, Length, alignment};
use iced_font_awesome::fa_icon;
use iced_modern_theme::Modern;


#[derive(Clone, Debug)]
pub enum Message {
    Dismiss(u32),
}

#[derive(Debug, Clone)]
pub struct ToastView {
    pub toast: Toast,
}

impl ToastView {
    pub fn new(toast: Toast) -> ToastView {
        ToastView { toast }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        // Botão de fechar alinhado à direita
        let close_button = button(
            Container::new(fa_icon("circle-xmark").size(17.5))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        )
        .width(Length::Fixed(25.0))
        .height(Length::Fixed(25.0))
        .on_press(Message::Dismiss(self.toast.id.expect("REASON")))
        .style(Modern::danger_button());

        // Header com botão à direita
        let header = Row::new()
            .width(Length::Fill)
            .push(Space::with_width(Length::Fill))
            .push(close_button);

        // Mensagem centralizada
        let message = Container::new(
            Text::new(&self.toast.message)
                .size(14)
                .style(Modern::primary_text()),
        )
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(Vertical::Center);

        // Layout principal com FillPortion
        let content = Column::new()
            .push(
                Container::new(header)
                    .height(Length::FillPortion(1))
                    .align_y(Vertical::Center),
            )
            .push(
                Container::new(message)
                    .height(Length::FillPortion(1))
                    .align_y(Vertical::Center),
            )
            .push(Space::with_height(Length::FillPortion(1)))
            .width(Length::Fill)
            .height(Length::Fill);

        Container::new(content)
            .padding(5)
            .width(Length::Fixed(300.0))
            .height(Length::Fixed(85.0))
            .style(Modern::sheet_container())
            .into()
    }
}
