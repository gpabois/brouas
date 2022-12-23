use self::{node_type::BPTreeNodeType, node_header::BPTreeHeader};

use super::{traits::Pager, PagerResult, id::PageId, page_type::PageType, offset::{PageOffset, PAGE_BODY_OFFSET}};

pub mod node_type;
pub mod node_header;
pub mod branch;
pub mod leaf;

pub type BPTreeNodeCellCapacity = u8;
pub struct BPTreeNodeOffset(u64);
pub struct BPTreeNodeHeaderOffset(u64);
pub struct BPTreeNodeBodyOffset(u64);

const BP_TREE_OFFSET: BPTreeNodeOffset = BPTreeNodeOffset(PAGE_BODY_OFFSET.const_into());
const BP_TREE_HEADER_OFFSET: BPTreeNodeHeaderOffset = BPTreeNodeHeaderOffset(BP_TREE_OFFSET.0);
const BP_TREE_BODY_OFFSET: BPTreeNodeBodyOffset = BPTreeNodeBodyOffset(BP_TREE_HEADER_OFFSET.0);

impl Into<PageOffset> for BPTreeNodeHeaderOffset {
    fn into(self) -> PageOffset {
        unsafe {
            PageOffset::new(self.0)
        }
    }
}

impl BPTreeNodeBodyOffset {
    pub const fn const_into(self) -> u64 {
        self.0
    }
}

pub struct BPTreeNode();

impl BPTreeNode 
{
    pub fn new<P>(pager: &mut P, node_type: BPTreeNodeType, capacity: u8) -> PagerResult<PageId> 
    where P: Pager {
        let page_id = pager.new_page(PageType::BTree)?;
        let header = BPTreeHeader::new(node_type, capacity);
        unsafe {
            pager.write_to_page(&page_id, &header, BP_TREE_HEADER_OFFSET)?;
        }
        Ok(page_id)
    }
}