use std::collections::HashSet;
use crate::dtos::tag_dto::TagDTO;

#[derive(Debug, Clone)]
pub struct ImageDTO {
    pub id: i64,
    pub path: String,
    pub thumbnail_path: String,
    pub description: String,
    pub tags: HashSet<TagDTO>,
    pub created_at: String,
    pub is_folder: bool,
    pub is_prepared: bool,
}

#[derive(Debug, Clone)]
pub struct ImageUpdateDTO {
    pub path: Option<String>,
    pub thumbnail_path: Option<String>,
    pub description: Option<String>,
    pub tags: Option<HashSet<TagDTO>>,
    pub is_folder: bool,
    pub is_prepared: bool,
}

impl Default for ImageUpdateDTO {
    fn default() -> Self {
        Self {
            path: None,
            thumbnail_path: None,
            description: None,
            tags: None,
            is_folder: false,
            is_prepared: false,
        }
    }
}

