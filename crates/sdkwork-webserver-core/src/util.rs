//! Shared Web helpers.

pub fn normalize_pagination(page: i32, page_size: i32) -> (i32, i32) {
    let page = if page < 1 { 1 } else { page };
    let page_size = page_size.clamp(1, 100);
    (page, page_size)
}

pub fn pagination_offset(page: i32, page_size: i32) -> i64 {
    let (page, page_size) = normalize_pagination(page, page_size);
    ((page - 1) as i64) * page_size as i64
}
