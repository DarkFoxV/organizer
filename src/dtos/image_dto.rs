use std::collections::HashSet;
use crate::dtos::tag_dto::TagDTO;

#[derive(Debug, Clone)]
pub struct ImageDTO {
    pub id: i64,
    pub path: String,
    pub thumbnail_path: String,
    pub description: String,
    pub tags: HashSet<TagDTO>,
}

#[derive(Debug, Clone)]
pub struct ImageUpdateDTO {
    pub path: Option<String>,
    pub thumbnail_path: Option<String>,
    pub description: Option<String>,
    pub tags: Option<HashSet<TagDTO>>,
}

impl Default for ImageUpdateDTO {
    fn default() -> Self {
        Self {
            path: None,
            thumbnail_path: None,
            description: None,
            tags: None,
        }
    }
}

