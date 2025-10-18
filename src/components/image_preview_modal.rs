use iced::widget::image::{viewer, Handle};
use iced::widget::{button, Column, Container, Row, Space, Text};
use iced::{Alignment, Background, Border, Color, Length, Shadow, Theme, Vector};
use iced::alignment::{Horizontal, Vertical};
use iced_font_awesome::fa_icon_solid;
use iced_modern_theme::Modern;

pub struct PreviewConfig<M> {
    pub handle: Handle,
    pub current_index: usize,
    pub total_images: usize,
    pub on_close: M,
    pub on_previous: Option<M>,
    pub on_next: Option<M>,
}

pub fn image_preview_modal<'a, M: 'a + Clone>(
    config: PreviewConfig<M>,
) -> iced::Element<'a, M> {
    let image_counter = format!("{} / {}", config.current_index + 1, config.total_images);

    let header: Row<_> = Row::new()
        .width(Length::Fill)
        .align_y(Vertical::Center)
        .push(
            Text::new(image_counter)
                .size(16)
                .style(Modern::secondary_text()),
        )
        .push(Space::with_width(Length::Fill))
        .push(
            button(
                Container::new(fa_icon_solid("xmark").size(24.0))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
                .width(Length::Fixed(40.0))
                .height(Length::Fixed(40.0))
                .on_press(config.on_close)
                .style(Modern::danger_button()),
        );

    let mut prev_button = button(
        Container::new(fa_icon_solid("chevron-left").size(24.0))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
        .width(Length::Fixed(50.0))
        .height(Length::Fixed(50.0))
        .style(Modern::secondary_button());

    if let Some(on_prev) = config.on_previous {
        prev_button = prev_button.on_press(on_prev);
    }

    let mut next_button = button(
        Container::new(fa_icon_solid("chevron-right").size(24.0))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center),
    )
        .width(Length::Fixed(50.0))
        .height(Length::Fixed(50.0))
        .style(Modern::secondary_button());

    if let Some(on_next) = config.on_next {
        next_button = next_button.on_press(on_next);
    }

    let body_with_navigation = Row::new()
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(Alignment::Center)
        .push(
            Container::new(prev_button)
                .width(Length::Fixed(70.0))
                .height(Length::Fill)
                .align_y(Alignment::Center)
                .padding([0, 10]),
        )
        .push(
            Container::new(
                viewer(config.handle)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center),
        )
        .push(
            Container::new(next_button)
                .width(Length::Fixed(70.0))
                .height(Length::Fill)
                .align_y(Alignment::Center)
                .padding([0, 10]),
        );

    let modal_content: Column<_> = Column::new()
        .spacing(15)
        .align_x(Horizontal::Center)
        .push(header)
        .push(body_with_navigation);

    Container::new(modal_content)
        .padding(30)
        .width(Length::FillPortion(9))
        .height(Length::FillPortion(9))
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .style(|theme: &Theme| iced::widget::container::Style {
            background: Some(Background::Color(theme.palette().background)),
            border: Border {
                color: Default::default(),
                width: 0.0,
                radius: 10.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: Vector::new(0.0, 8.0),
                blur_radius: 16.0,
            },
            ..Default::default()
        })
        .into()
}