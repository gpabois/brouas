use crate::{pager::{PagerResult, id::PageId, TraitPager}, io::traits::{OutStream, InStream}};
use super::{node_type::BPTreeNodeType, BPTreeCellId, BPTreeNodeOffset};

pub trait BPTreeNode 
{
    fn new<P>(pager: &mut P, node_type: BPTreeNodeType, capacity: u8) -> PagerResult<PageId> 
    where P: TraitPager;

    fn insert_cell<P>(pager: &mut P, page_id: &PageId, cell_id: &BPTreeCellId) -> PagerResult<()>
    where P: TraitPager;
    
    fn remove_cell<P>(pager: &mut P, page_id: &PageId, cell_id: &BPTreeCellId) -> PagerResult<()>
    where P: TraitPager;

    fn write_cell<P, D>(pager: &mut P, page_id: &PageId, cell_id: &BPTreeCellId, cell_offset: &BPTreeNodeOffset, cell: &D) -> PagerResult<()>
    where P: TraitPager, D: OutStream;

    fn read_cell<P, D>(pager: &P, page_id: &PageId, cell_id: &BPTreeCellId, cell_offset: &BPTreeNodeOffset, cell: &mut D) -> PagerResult<()>
    where P: TraitPager, D: InStream;
}