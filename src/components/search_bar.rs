use iced::widget::{Button, Container, PickList, Row, Text, TextInput};
use iced::{Alignment, Length};
use iced::alignment::{Horizontal, Vertical};
use iced_font_awesome::fa_icon_solid;
use iced_modern_theme::Modern;

pub struct SearchBarConfig<'a, M, T: Clone + PartialEq> {
    pub query: &'a str,
    pub sort_order: T,
    pub sort_options: &'a [T],
    pub on_query_change: Box<dyn Fn(String) -> M + 'a>,
    pub on_search: M,
    pub on_register: M,
    pub on_sort_change: Box<dyn Fn(T) -> M + 'a>,
}

pub fn search_bar<'a, M: 'a + Clone, T: 'a + Clone + PartialEq + std::fmt::Display>(
    config: SearchBarConfig<'a, M, T>,
) -> iced::Element<'a, M> {
    Container::new(
        Row::new()
            .spacing(15)
            .push(
                Container::new(
                    TextInput::new(t!("search.input.description").as_ref(), config.query)
                        .on_input(config.on_query_change)
                        .on_submit(config.on_search.clone())
                        .style(Modern::search_input())
                        .padding([12, 16])
                        .size(16),
                )
                    .width(Length::FillPortion(5)),
            )
            .push(
                Button::new(
                    Container::new(
                        Row::new()
                            .spacing(8)
                            .align_y(Alignment::Center)
                            .push(fa_icon_solid("magnifying-glass").size(18.0))
                            .push(Text::new(t!("search.button.search")).size(16)),
                    )
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                )
                    .style(Modern::primary_button())
                    .on_press(config.on_search)
                    .width(Length::FillPortion(2))
                    .padding([12, 20]),
            )
            .push(
                Button::new(
                    Container::new(
                        Row::new()
                            .spacing(8)
                            .align_y(Alignment::Center)
                            .push(fa_icon_solid("plus").size(18.0))
                            .push(Text::new(t!("search.button.register")).size(16)),
                    )
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                )
                    .style(Modern::success_button())
                    .on_press(config.on_register)
                    .width(Length::FillPortion(2))
                    .padding([12, 20]),
            )
            .push(
                Container::new(
                    PickList::new(
                        config.sort_options,
                        Some(config.sort_order),
                        config.on_sort_change,
                    )
                        .style(Modern::pick_list())
                        .padding([12, 16])
                        .text_size(16),
                )
                    .width(Length::FillPortion(1)),
            ),
    )
        .width(Length::Fill)
        .padding(20)
        .style(Modern::card_container())
        .into()
}