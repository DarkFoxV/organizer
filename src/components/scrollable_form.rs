use iced::widget::{Column, Scrollable, Space};
use iced::{Element, Length};

pub struct ScrollableFormConfig<'a, M> {
    pub header: Element<'a, M>,
    pub content_section: Element<'a, M>,
    pub description_section: Element<'a, M>,
    pub tags_section: Element<'a, M>,
    pub bottom_section: Element<'a, M>,
}

pub fn scrollable_form<'a, M: 'a>(
    config: ScrollableFormConfig<'a, M>,
) -> Column<'a, M> {  // Mudou aqui
    Column::new()
        .spacing(20)
        .push(config.header)
        .push(
            Scrollable::new(
                Column::new()
                    .padding(20)
                    .spacing(20)
                    .push(config.content_section)
                    .push(config.description_section)
                    .push(config.tags_section)
                    .push(Space::with_height(20))
                    .push(config.bottom_section),
            )
                .width(Length::Fill)
                .height(Length::Fill),
        )
}