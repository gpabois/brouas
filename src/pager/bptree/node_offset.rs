use crate::pager::offset::PageOffset;

#[derive(Clone, Copy)]
/// Offset from the node body base.
pub struct BPTreeNodeOffset(u64);

impl std::ops::Add<BPTreeNodeOffset> for BPTreeNodeOffset {
    type Output = BPTreeNodeOffset;

    fn add(self, rhs: BPTreeNodeOffset) -> Self::Output {
        BPTreeNodeOffset(self.0 + rhs.0)
    }
}

impl From<u64> for BPTreeNodeOffset {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

/// Represent a node cell capacity (max number of cells)
impl std::ops::Add<PageOffset> for BPTreeNodeOffset 
{
    type Output = PageOffset;

    fn add(self, rhs: PageOffset) -> Self::Output 
    {
        rhs + self.0
    }
}


impl BPTreeNodeOffset {
    pub fn new(value : u64) -> Self {
        Self(value)
    } 
    pub fn to_page_offset(self, base: &PageOffset) -> PageOffset {
        self + base
    }
}