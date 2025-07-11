use iced::widget::{Button, Row, Text};
use iced::Element;
use iced_modern_theme::Modern;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum Message {
    ToggleTag(String),
}

#[derive(Debug, Clone)]
pub struct TagSelector {
    pub selected: HashSet<String>,
    pub available: Vec<String>,
}

impl TagSelector {
    pub fn new(available: Vec<String>) -> Self {
        Self {
            selected: HashSet::new(),
            available,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let mut row = Row::new().spacing(10);

        for tag in &self.available {
            let selected = self.selected.contains(tag);

            let button = if selected {
                Button::new(Text::new(Self::capitalize_first(&tag)))
                    .style(Modern::green_tinted_button())
                    .padding(5)
                    .on_press(Message::ToggleTag(tag.clone()))
            } else {
                Button::new(Text::new(Self::capitalize_first(&tag)))
                    .style(Modern::blue_tinted_button())
                    .padding(5)
                    .on_press(Message::ToggleTag(tag.clone()))
            };

            row = row.push(button);
        }

        row.wrap().into()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::ToggleTag(tag) => {
                if self.selected.contains(&tag) {
                    self.selected.remove(&tag);
                } else {
                    self.selected.insert(tag);
                }
            }
        }
    }

    pub fn selected_tags(&self) -> HashSet<String> {
        self.selected.iter().cloned().collect()
    }

    fn capitalize_first(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        }
    }

}
