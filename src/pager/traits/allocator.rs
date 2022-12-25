use crate::pager::{result::PagerResult, id::PageId};

use super::pager::Pager;

pub trait Allocator {
    fn alloc<P: Pager>(pager: &mut P) -> PagerResult<Option<PageId>>;
    fn free<P: Pager>(pager: &mut P, page_id: &PageId) -> PagerResult<()>;
}