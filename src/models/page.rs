#[derive(Debug, Clone)]
pub struct Page<T> {
    pub content: Vec<T>,
    pub total_pages: u64,
    pub page_number: u64,
}
