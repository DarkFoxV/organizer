use iced::widget::{Button, Container, Row, Text};
use iced::{Alignment, Length};
use iced::alignment::{Horizontal, Vertical};
use iced_font_awesome::fa_icon_solid;
use iced_modern_theme::Modern;

pub fn pagination<'a, M: 'a + Clone>(
    current_page: u64,
    total_pages: u64,
    on_page_change: impl Fn(u64) -> M + 'a + Copy,
) -> iced::Element<'a, M> {
    if total_pages <= 1 {
        return Container::new(Text::new(""))
            .width(Length::Fixed(0.0))
            .height(Length::Fixed(0.0))
            .into();
    }

    let mut pagination_row = Row::new().spacing(8).align_y(Alignment::Center);

    // Previous button
    if current_page > 0 {
        pagination_row = pagination_row.push(
            Button::new(
                Container::new(
                    Row::new()
                        .spacing(6)
                        .align_y(Alignment::Center)
                        .push(fa_icon_solid("chevron-left").size(14.0))
                        .push(Text::new(t!("search.button.previous")).size(14)),
                )
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center),
            )
                .style(Modern::secondary_button())
                .on_press(on_page_change(current_page - 1))
                .padding([8, 12]),
        );
    }

    let start_page = if current_page > 2 {
        current_page - 2
    } else {
        0
    };
    let end_page = std::cmp::min(start_page + 5, total_pages);

    // First page + ellipsis
    if start_page > 0 {
        pagination_row = pagination_row.push(
            Button::new(Text::new("1").size(14))
                .style(Modern::blue_tinted_button())
                .on_press(on_page_change(0))
                .padding([8, 12]),
        );
        if start_page > 1 {
            pagination_row = pagination_row
                .push(Text::new("...").size(14).style(Modern::secondary_text()));
        }
    }

    // Page numbers
    for page_index in start_page..end_page {
        let label = (page_index + 1).to_string();
        let is_current = page_index == current_page;

        let button = if is_current {
            Button::new(Text::new(label).size(14))
                .style(Modern::primary_button())
                .padding([8, 12])
        } else {
            Button::new(Text::new(label).size(14))
                .style(Modern::blue_tinted_button())
                .on_press(on_page_change(page_index))
                .padding([8, 12])
        };

        pagination_row = pagination_row.push(button);
    }

    // Ellipsis + last page
    if end_page < total_pages {
        if end_page < total_pages - 1 {
            pagination_row = pagination_row
                .push(Text::new("...").size(14).style(Modern::secondary_text()));
        }
        pagination_row = pagination_row.push(
            Button::new(Text::new(total_pages.to_string()).size(14))
                .style(Modern::blue_tinted_button())
                .on_press(on_page_change(total_pages - 1))
                .padding([8, 12]),
        );
    }

    // Next button
    if current_page < total_pages - 1 {
        pagination_row = pagination_row.push(
            Button::new(
                Container::new(
                    Row::new()
                        .spacing(6)
                        .align_y(Alignment::Center)
                        .push(Text::new(t!("search.button.next")).size(14))
                        .push(fa_icon_solid("chevron-right").size(14.0)),
                )
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center),
            )
                .style(Modern::secondary_button())
                .on_press(on_page_change(current_page + 1))
                .padding([8, 12]),
        );
    }

    Container::new(pagination_row)
        .width(Length::Shrink)
        .align_x(Horizontal::Center)
        .padding(20)
        .into()
}