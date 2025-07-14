use crate::models::tag_color::TagColor;

#[derive(Debug, Clone)]
#[derive(Eq, Hash, PartialEq)]
pub struct TagDTO {
    pub id: i64,
    pub name: String,
    pub color: TagColor,
}

#[derive(Debug, Clone)]
pub struct TagUpdateDTO {
    pub name: String,
    pub color: TagColor,
}

impl Default for TagUpdateDTO {
    fn default() -> Self {
        TagUpdateDTO {
            name: String::new(),
            color: TagColor::default(),
        }
    }
}