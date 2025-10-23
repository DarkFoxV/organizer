use std::fmt;

#[derive(Clone, Debug)]
#[derive(PartialEq)]
pub enum ImageType {
    Folder,
    Image,
    FromFolder,
}

impl ImageType {
    pub fn from_str(s: &str) -> ImageType {
        match s {
            "folder" => ImageType::Folder,
            "image" => ImageType::Image,
            "from_folder" => ImageType::FromFolder,
            _ => ImageType::Image,
        }
    }
}

impl fmt::Display for ImageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ImageType::Folder => "folder",
            ImageType::Image => "image",
            ImageType::FromFolder => "from_folder",
        };
        write!(f, "{s}")
    }
}
