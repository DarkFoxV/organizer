use crate::screen::search::Message;

use iced::Length;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::image::Handle;
use iced::widget::{Button, Column, Container, Image, Row, Scrollable, Text};
use iced_font_awesome::fa_icon_solid;
use iced_modern_theme::Modern;
use crate::dtos::image_dto::ImageDTO;

#[derive(Debug, Clone)]
pub struct ImageContainer {
    pub id: i64,
    pub image_dto: ImageDTO,
    pub handle: Handle,
}

impl ImageContainer {
    pub fn new(image_data: ImageDTO) -> Self {
        let handle = Handle::from_path(image_data.thumbnail_path.clone());
        Self {
            id: image_data.id,
            image_dto: image_data,
            handle,
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        let description =
            Scrollable::new(Text::new(&self.image_dto.description).align_x(Horizontal::Center))
                .height(Length::FillPortion(4));

        let image_widget = Image::new(&self.handle)
            .width(Length::Fill)
            .height(Length::FillPortion(6));

        let created_at = Text::new(&self.image_dto.created_at)
            .size(12)
            .style(Modern::secondary_text());

        let buttons = Row::new()
            .spacing(10)
            .push(
                Button::new(
                    Container::new(fa_icon_solid("eraser").size(25.0))
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                )
                .style(Modern::danger_button())
                .width(Length::FillPortion(1))
                .on_press(Message::DeleteImage(self.image_dto.clone())),
            )
            .push(
                Button::new(
                    Container::new(fa_icon_solid("clipboard").size(25.0))
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                )
                .style(Modern::primary_button())
                .width(Length::FillPortion(1))
                .on_press(Message::CopyImage(self.image_dto.path.clone())),
            )
            .push(
                Button::new(
                    Container::new(fa_icon_solid("book-open").size(25.0))
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                )
                .style(Modern::success_button())
                .width(Length::FillPortion(1))
                .on_press(Message::OpenImage(self.id)),
            )
            .push(
                Button::new(
                    Container::new(fa_icon_solid("file-pen").size(25.0))
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                )
                .style(Modern::warning_button())
                .width(Length::FillPortion(1))
                .on_press(Message::Update(self.image_dto.clone())),
            )
            .push(
                Button::new(
                    Container::new(fa_icon_solid("file").size(25.0))
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                )
                .style(Modern::system_button())
                .width(Length::FillPortion(1))
                .on_press(Message::OpenLocalImage(self.id)),
            );

        let buttons_container = Container::new(buttons)
            .align_x(Horizontal::Center)
            .height(Length::FillPortion(1));

        Container::new(
            Column::new()
                .spacing(10)
                .push(image_widget)
                .push(description)
                .push(created_at)
                .push(buttons_container),
        )
        .padding(10)
        .width(Length::Fixed(220.0))
        .height(Length::Fixed(400.0))
        .style(Modern::accent_container())
        .into()
    }
}
