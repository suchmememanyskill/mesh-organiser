pub struct PaginatedResponse<T> {
    pub page: u32,
    pub page_size: u32,
    pub items: Vec<T>,
}