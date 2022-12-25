use crate::pager::offset::PageOffset;

use self::header::PAGE_HEADER_SIZE;

pub mod header;

pub const PAGE_BODY_OFFSET: PageOffset = header::PAGE_HEADER_OFFSET + header::PageHeader::size_of();
pub const MIN_PAGE_SIZE: u64 = const_utils::u64::max(PAGE_HEADER_SIZE, PAGE_HEADER_SIZE);