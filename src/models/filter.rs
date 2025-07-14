use std::collections::HashSet;
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)] // <-- adicione Copy e Eq
pub enum SortOrder {
    CreatedAsc,
    CreatedDesc,
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortOrder::CreatedAsc => write!(f, "{}", t!("search.order.oldest")),
            SortOrder::CreatedDesc => write!(f, "{}", t!("search.order.newest")),
        }
    }
}

pub struct Filter {
    pub query: String,
    pub tags: HashSet<String>,
    pub sort_order: SortOrder,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            tags: HashSet::new(),
            sort_order: SortOrder::CreatedDesc,
        }
    }
}
