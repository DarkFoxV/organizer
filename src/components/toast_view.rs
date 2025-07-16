use crate::models::toast::{Toast, ToastKind};
use iced::alignment::Vertical;
use iced::widget::{button, Container, Row, Space, Text};
use iced::{alignment, Alignment, Background, Border, Color, Element, Length, Padding, Shadow, Vector};
use iced_font_awesome::fa_icon_solid;

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

    pub fn view(&self) -> Element<'_, Message> {
        // Toast Colors
        let (bg_color, border_color, icon_name, icon_color, text_color) = match self.toast.kind {
            ToastKind::Success => (
                Color::from_rgb(0.9, 0.98, 0.9),
                Color::from_rgb(0.2, 0.8, 0.2),
                "circle-check",
                Color::from_rgb(0.2, 0.8, 0.2),
                Color::from_rgb(0.1, 0.5, 0.1),
            ),
            ToastKind::Error => (
                Color::from_rgb(0.98, 0.9, 0.9),
                Color::from_rgb(0.9, 0.2, 0.2),
                "circle-exclamation",
                Color::from_rgb(0.9, 0.2, 0.2),
                Color::from_rgb(0.7, 0.1, 0.1),
            ),
            ToastKind::Warning => (
                Color::from_rgb(0.99, 0.96, 0.9),
                Color::from_rgb(0.9, 0.7, 0.2),
                "triangle-exclamation",
                Color::from_rgb(0.9, 0.7, 0.2),
                Color::from_rgb(0.7, 0.5, 0.1),
            ),
            ToastKind::Info => (
                Color::from_rgb(0.9, 0.95, 0.99),
                Color::from_rgb(0.2, 0.6, 0.9),
                "circle-info",
                Color::from_rgb(0.2, 0.6, 0.9),
                Color::from_rgb(0.1, 0.4, 0.7),
            ),
        };

        let status_icon = Container::new(
            fa_icon_solid(icon_name)
                .size(20.0)
                .color(icon_color),
        )
            .width(Length::Fixed(30.0))
            .height(Length::Fixed(30.0))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center);

        let message_text = Container::new(
            Text::new(&self.toast.message)
                .size(15)
                .color(text_color),
        )
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Left)
            .align_y(Vertical::Center)
            .padding(Padding::from([0, 10]));

        let close_button = button(
            Container::new(
                fa_icon_solid("xmark")
                    .size(16.0)
                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
            )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        )
            .width(Length::Fixed(28.0))
            .height(Length::Fixed(28.0))
            .on_press(Message::Dismiss(self.toast.id.expect("Toast ID is required")))
            .style(|_, _| button::Style {
                background: Some(Background::Color(Color::TRANSPARENT)),
                text_color: Color::from_rgb(0.6, 0.6, 0.6),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: iced::border::Radius::from(14.0),
                },
                shadow: Shadow::default(),
            });

        let color_bar = Container::new(Space::with_width(Length::Fixed(4.0)))
            .height(Length::Fill)
            .style(move |_theme: &iced::Theme| iced::widget::container::Style {
                background: Some(Background::Color(border_color)),
                border: Border::default(),
                shadow: Shadow::default(),
                text_color: None,
            });

        let main_content = Row::new()
            .spacing(0)
            .push(color_bar)
            .push(
                Row::new()
                    .spacing(12)
                    .padding(Padding::from([15, 20]))
                    .align_y(Alignment::Center)
                    .push(status_icon)
                    .push(message_text)
                    .push(close_button)
                    .width(Length::Fill),
            );

        Container::new(main_content)
            .width(Length::Fixed(350.0))
            .height(Length::Fixed(75.0))
            .style(move |_theme: &iced::Theme| iced::widget::container::Style {
                background: Some(Background::Color(bg_color)),
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: iced::border::Radius::from(12.0),
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                    offset: Vector::new(0.0, 4.0),
                    blur_radius: 12.0,
                },
                text_color: None,
            })
            .into()
    }
}