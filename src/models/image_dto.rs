use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ImageDTO {
    pub id: i64,
    pub path: String,
    pub thumbnail_path: String,
    pub description: String,
    pub tags: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct ImageUpdateDTO {
    pub path: Option<String>,
    pub thumbnail_path: Option<String>,
    pub description: Option<String>,
    pub tags: Option<HashSet<String>>,
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

