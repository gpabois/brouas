use crate::io::traits::OutStream;
use crate::pager::bptree::cell_capacity::BPTreeCellCapacity;
use crate::pager::bptree::cell_id::BPTreeCellId;
use crate::pager::bptree::cell_size::BPTreeCellSize;
use crate::pager::id::PageId;
use crate::pager::result::PagerResult;
use crate::pager::traits::pager::Pager;

pub trait BPTreeLeafCell 
{
    type Key: Into<u64>;
    type Element: OutStream;

    fn borrow_element(&self) -> &Self::Element;
    fn borrow_mut_element(&mut self) -> &Self::Element;
    fn get_key(&self) -> Self::Key;
}

pub trait BPTreeLeaf
{
    fn new<P, C>(pager: &mut P, capacity: BPTreeCellCapacity, cell: &C) -> PagerResult<PageId> 
    where P: Pager, C: BPTreeLeafCell;

    fn find_nearest_cell_by_key<P>(pager: &P, page_id: &PageId, key: u64) -> PagerResult<Option<BPTreeCellId>>
    where P: Pager;
    
    fn find_cell_by_key<P>(pager: &P, page_id: &PageId, key: u64) -> PagerResult<Option<BPTreeCellId>>
    where P: Pager;

    fn insert<P, C>(pager: &mut P, page_id: &PageId, cell: &C) -> PagerResult<()> 
    where P: Pager, C: BPTreeLeafCell;

    fn remove_by_key<P>(pager: &mut P, page_id: &PageId, key: impl Into<u64>) -> PagerResult<()>
    where P: Pager;

    fn max_in_page_element_size(size: &BPTreeCellSize) -> u64;
}