use sea_orm::entity::prelude::*;
use sea_orm::Iterable;
use sea_orm::EnumIter;

#[derive(Clone, Debug, PartialEq, Eq, Hash, DeriveActiveEnum, EnumIter)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum TagColor {
    #[sea_orm(string_value = "red")]
    Red,
    #[sea_orm(string_value = "green")]
    Green,
    #[sea_orm(string_value = "blue")]
    Blue,
    #[sea_orm(string_value = "orange")]
    Orange,
    #[sea_orm(string_value = "white")]
    White,
}

impl Default for TagColor {
    fn default() -> Self {
        TagColor::Blue
    }
}

impl TagColor {
    pub fn as_str(&self) -> &'static str {
        match self {
            TagColor::Red => "red",
            TagColor::Green => "green",
            TagColor::Blue => "blue",
            TagColor::Orange => "orange",
            TagColor::White => "white",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "red" => Some(TagColor::Red),
            "green" => Some(TagColor::Green),
            "blue" => Some(TagColor::Blue),
            "orange" => Some(TagColor::Orange),
            "white" => Some(TagColor::White),
            _ => None,
        }
    }

    pub fn all() -> Vec<TagColor> {
        TagColor::iter().collect()
    }
}