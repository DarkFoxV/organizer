use iced::{Alignment, Length};
use iced::widget::{button, Container, Row, Space};
use iced_font_awesome::fa_icon_solid;
use iced_modern_theme::Modern;

pub fn header<'a, M: 'a + Clone, F>(on_close: F) -> iced::Element<'a, M>
where
    F: Fn() -> M + 'a + Copy,
{

    Container::new(
        Row::new()
            .width(Length::Fill)
            .align_y(Alignment::Center)
            .push(Space::with_width(Length::Fill))
            .push(
                button(
                    Container::new(fa_icon_solid("xmark").size(20.0))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                    .width(Length::Fixed(40.0))
                    .height(Length::Fixed(40.0))
                    .on_press(on_close())
                    .style(Modern::danger_button()),
            ),
    )
        .padding(iced::Padding {
            top: 10.0,
            right: 22.5,
            bottom: 0.0,
            left: 22.5,
        })
        .width(Length::Fill)
        .into()
}
