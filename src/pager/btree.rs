use self::{node_type::BPTreeNodeType, node_header::BPTreeHeader};

use super::{traits::Pager, PagerResult, id::PageId, page_type::PageType, offset::{PageOffset, PAGE_BODY_OFFSET}};

pub mod node_type;
pub mod node_header;
pub mod branch;
pub mod leaf;

pub type BPTreeNodeCellCapacity = u8;

const BP_TREE_OFFSET: PageOffset = PAGE_BODY_OFFSET;
const BP_TREE_HEADER_OFFSET: PageOffset = BP_TREE_OFFSET;
const BP_TREE_BODY_OFFSET: PageOffset = BP_TREE_HEADER_OFFSET;

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