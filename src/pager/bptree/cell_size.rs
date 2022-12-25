use crate::pager::{page::PageSize, offset::PageOffset};
use super::{cell_capacity::BPTreeCellCapacity, cell_id::BPTreeCellId, node_offset::BPTreeNodeOffset};

#[derive(Copy, Clone)]
pub struct BPTreeCellSize(u64);

impl BPTreeCellSize
{
    pub const fn from(page_size: PageSize, bptree_body_offset: PageOffset, capacity: BPTreeCellCapacity) -> Self {
        Self(Self::raw_cell_size(page_size, bptree_body_offset, capacity))
    }

    const fn raw_cell_size(page_size: PageSize, bptree_body_offset: PageOffset, capacity: BPTreeCellCapacity) -> u64 {
        let body_size = page_size.wrapping_sub(bptree_body_offset);
        let cell_size = body_size / (capacity as u64);
        cell_size    
    }
}

impl<T> std::ops::Mul<T> for BPTreeCellSize
where T: Into<BPTreeCellId> 
{
    type Output = BPTreeNodeOffset;

    fn mul(self, rhs: T) -> Self::Output {
        BPTreeNodeOffset::new((rhs.into().0 as u64) * self.0)
    }
}