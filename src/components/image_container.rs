use crate::dtos::image_dto::ImageDTO;
use crate::screen::search::Message;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::image::Handle;
use iced::widget::tooltip::Position;
use iced::widget::{Button, Column, Container, Image, Row, Scrollable, Text, Tooltip};
use iced::{Background, Border, Color, Length, Shadow, Theme, Vector};
use iced_font_awesome::fa_icon_solid;
use iced_modern_theme::Modern;
use crate::models::enums::image_type::ImageType;

#[derive(Debug, Clone)]
pub struct ImageContainer {
    pub id: i64,
    pub image_dto: ImageDTO,
    pub handle: Handle,
    pub is_from_folder: bool,

    pub tooltip_delete: String,
    pub tooltip_edit: String,
    pub tooltip_view: String,
    pub tooltip_copy: String,
    pub tooltip_open_local: String,
}

impl ImageContainer {
    pub fn new(image_data: ImageDTO, is_from_folder: bool) -> Self {
        let handle = Handle::from_path(image_data.thumbnail_path.clone());
        Self {
            id: image_data.id,
            image_dto: image_data,
            handle,
            is_from_folder,
            tooltip_delete: t!("message.image.container.delete").to_string(),
            tooltip_edit: t!("message.image.container.edit").to_string(),
            tooltip_view: t!("message.image.container.open").to_string(),
            tooltip_copy: t!("message.image.container.copy").to_string(),
            tooltip_open_local: t!("message.image.container.open_local").to_string(),
        }
    }

    pub fn view(&'_ self) -> iced::Element<'_, Message> {
        let image_widget = if self.image_dto.is_prepared {
            Container::new(
                Image::new(&self.handle)
                    .width(Length::Fill)
                    .height(Length::Fixed(180.0)),
            )
            .padding(8)
            .width(Length::Fill)
            .height(Length::Fixed(180.0))
        } else {
            Container::new(fa_icon_solid("hourglass-half").size(32.0))
                .padding(8)
                .width(Length::Fill)
                .height(Length::Fixed(180.0))
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
        };

        let description = Container::new(Scrollable::new(
            Container::new(
                Text::new(&self.image_dto.description)
                    .size(14)
                    .style(Modern::primary_text()),
            )
            .padding([8, 12])
            .width(Length::Fill),
        ))
        .height(Length::Fixed(90.0))
        .width(Length::Fill);

        let created_at = Container::new(
            Text::new(&self.image_dto.created_at)
                .size(11)
                .style(Modern::secondary_text()),
        )
        .width(Length::Fill)
        .align_x(Horizontal::Center)
        .padding([4, 8]);

        let image_type = if self.is_from_folder {
            ImageType::FromFolder
        } else if self.image_dto.is_folder {
            ImageType::Folder
        } else {
            ImageType::Image
        };

        let delete_message = Message::DeleteImage(self.image_dto.clone(), image_type);

        let delete_button: iced::Element<Message> = Tooltip::new(
            Button::new(
                Container::new(fa_icon_solid("trash").size(16.0))
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .style(Modern::danger_button())
            .width(Length::FillPortion(1))
            .height(Length::Fixed(36.0))
            .on_press(delete_message),
            self.tooltip_delete.as_str(),
            Position::Top,
        )
        .style(Modern::card_container())
        .padding(8)
        .gap(4)
        .into();

        let copy_button = if !self.image_dto.is_folder {
            Some(
                Tooltip::new(
                    Button::new(
                        Container::new(fa_icon_solid("copy").size(16.0))
                            .align_x(Horizontal::Center)
                            .align_y(Vertical::Center)
                            .width(Length::Fill)
                            .height(Length::Fill),
                    )
                    .style(Modern::primary_button())
                    .width(Length::FillPortion(1))
                    .height(Length::Fixed(36.0))
                    .on_press(Message::CopyImage(self.image_dto.path.clone())),
                    self.tooltip_copy.as_str(),
                    Position::Top,
                )
                .style(Modern::card_container())
                .padding(8)
                .gap(4),
            )
        } else {
            None
        };

        let view_button = Tooltip::new(
            Button::new(
                Container::new(fa_icon_solid("eye").size(16.0))
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .style(Modern::success_button())
            .width(Length::FillPortion(1))
            .height(Length::Fixed(36.0))
            .on_press(Message::OpenImage(self.image_dto.clone())),
            self.tooltip_view.as_str(),
            Position::Top,
        )
        .style(Modern::card_container())
        .padding(8)
        .gap(4);

        let edit_button = if !self.is_from_folder {
            Some(
                Tooltip::new(
                    Button::new(
                        Container::new(fa_icon_solid("pen-to-square").size(16.0))
                            .align_x(Horizontal::Center)
                            .align_y(Vertical::Center)
                            .width(Length::Fill)
                            .height(Length::Fill),
                    )
                    .style(Modern::warning_button())
                    .width(Length::FillPortion(1))
                    .height(Length::Fixed(36.0))
                    .on_press(Message::Update(self.image_dto.clone())),
                    self.tooltip_edit.as_ref(),
                    Position::Top,
                )
                .style(Modern::card_container())
                .padding(8)
                .gap(4),
            )
        } else {
            None
        };

        let open_local_button = Tooltip::new(
            Button::new(
                Container::new(fa_icon_solid("folder-open").size(16.0))
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .style(Modern::system_button())
            .width(Length::FillPortion(1))
            .height(Length::Fixed(36.0))
            .on_press(Message::OpenLocalImage(self.id)),
            self.tooltip_open_local.as_str(),
            Position::Top,
        )
        .style(Modern::card_container())
        .padding(8)
        .gap(4);

        let mut action_buttons = Row::new()
            .spacing(6)
            .push(delete_button)
            .push(view_button)
            .push(open_local_button);

        if let Some(edit_btn) = edit_button {
            action_buttons = action_buttons.push(edit_btn);
        }
        if let Some(copy_btn) = copy_button {
            action_buttons = action_buttons.push(copy_btn);
        }

        // Container dos botões
        let buttons_container = Container::new(action_buttons)
            .width(Length::Fill)
            .padding([8, 12]);

        // Layout principal do card
        let card_content = if self.image_dto.is_prepared {
            Column::new()
                .spacing(0)
                .push(image_widget)
                .push(description)
                .push(created_at)
                .push(buttons_container)
        } else {
            Column::new()
                .spacing(0)
                .push(image_widget)
                .push(description)
                .push(created_at)
        };

        // Card container com sombra e bordas arredondadas
        Container::new(card_content)
            .padding(5)
            .width(Length::Fixed(220.0))
            .height(Length::Fixed(360.0))
            .style(move |theme: &Theme| iced::widget::container::Style {
                background: Some(Background::Color(theme.palette().background)),
                border: Border {
                    color: if self.image_dto.is_folder {
                        Color::from_rgb(0.0, 0.5, 1.0) // Azul
                    }
                    else {
                        Color::from_rgba(0.0, 0.0, 0.0, 0.1)
                    },
                    width: 1.0,
                    radius: 12.0.into(),
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 8.0,
                },
                ..Default::default()
            })
            .into()

    }
}
