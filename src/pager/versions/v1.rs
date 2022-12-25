use crate::pager::traits::{pager::PagerVersion, stream::PagerStream};

use self::page::MIN_PAGE_SIZE;

pub mod bptree;
pub mod allocator;
pub mod overflow;
pub mod page;
pub mod pager;

pub struct V1;

impl PagerVersion for V1 
{
    type Allocator = allocator::Allocator;
    type PagerHeader = pager::header::PagerHeader;
    type PageHeader = page::header::PageHeader;

    const MIN_PAGE_SIZE: u64 = MIN_PAGE_SIZE;
}

pub type Pager<S: PagerStream> = crate::pager::Pager<V1, S>;