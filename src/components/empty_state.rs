use iced::widget::{Column, Container, Text};
use iced::{Alignment, Length};
use iced_font_awesome::fa_icon;
use iced_modern_theme::Modern;

pub fn empty_state<'a, M: 'a>(
    icon: &'a str,
    title: &'a str,
    subtitle: &'a str,
) -> iced::Element<'a, M> {
    let column = Column::new()
        .spacing(20)
        .align_x(Alignment::Center)
        .push(Container::new(fa_icon(icon).size(64.0)))
        .push(Text::new(title).size(18).style(Modern::secondary_text()))
        .push(Text::new(subtitle).size(14).style(Modern::secondary_text()));

    Container::new(column)
        .width(Length::Fill)
        .height(Length::Fixed(300.0))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into()
}