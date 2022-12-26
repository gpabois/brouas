use super::{header::{PAGE_HEADER_SIZE}};

pub type PageOffset = u64;

pub const PAGE_HEADER_OFFSET: PageOffset = 0;
pub const PAGE_BODY_OFFSET: PageOffset = PAGE_HEADER_OFFSET + PAGE_HEADER_SIZE;
