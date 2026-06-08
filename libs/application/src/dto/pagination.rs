use serde::Serialize;

/// Bao phản hồi danh sách kèm thông tin phân trang (dùng chung mọi list endpoint).
#[derive(Debug, Serialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub page: u32,
    pub page_size: u32,
    pub total_items: i64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}
