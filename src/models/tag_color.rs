use sea_orm::entity::prelude::*;
use sea_orm::EnumIter;
use sea_orm::Iterable;
use std::fmt;

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
    #[sea_orm(string_value = "purple")]
    Purple,
    #[sea_orm(string_value = "pink")]
    Pink,
    #[sea_orm(string_value = "indigo")]
    Indigo,
    #[sea_orm(string_value = "Teal")]
    Teal,
    #[sea_orm(string_value = "Gray")]
    Gray,
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
            TagColor::Purple => "purple",
            TagColor::Pink => "pink",
            TagColor::Indigo => "indigo",
            TagColor::Teal => "teal",
            TagColor::Gray => "gray",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "red" => Some(TagColor::Red),
            "green" => Some(TagColor::Green),
            "blue" => Some(TagColor::Blue),
            "orange" => Some(TagColor::Orange),
            "purple" => Some(TagColor::Purple),
            "pink" => Some(TagColor::Pink),
            "indigo" => Some(TagColor::Indigo),
            "teal" => Some(TagColor::Teal),
            "gray" => Some(TagColor::Gray),
            _ => None,
        }
    }

    pub fn all() -> Vec<TagColor> {
        TagColor::iter().collect()
    }

}

impl fmt::Display for TagColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TagColor::Red => t!("tag.color.red"),
            TagColor::Green => t!("tag.color.green"),
            TagColor::Blue => t!("tag.color.blue"),
            TagColor::Orange => t!("tag.color.orange"),
            TagColor::Purple => t!("tag.color.purple"),
            TagColor::Pink => t!("tag.color.pink"),
            TagColor::Indigo => t!("tag.color.indigo"),
            TagColor::Teal => t!("tag.color.teal"),
            TagColor::Gray => t!("tag.color.gray"),
        };
        write!(f, "{}", s)
    }
}
