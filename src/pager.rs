use std::{io::{Read, BufReader, Write, BufWriter}, alloc::{Layout, alloc_zeroed, dealloc}, mem::size_of, collections::HashMap, ops::Add};

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub struct PageId(u64);

impl Add<u64> for PageId
{
    type Output = PageId;

    fn add(self, rhs: u64) -> Self::Output {
        PageId(self.0 + rhs)
    }
}

pub struct PageHeader
{
    /// Number of the page
    id:         u64,
    /// Unique number, in case of conflicted pages
    nonce:      u16,
    /// Type of page, 1 = Collection Tree, 0 = B+ Tree, 1 = Overflow page
    page_type:  u8
}

pub struct TreePageHeader
{
    /// Type of node: 1 = Leaf, 0 = Branch
    node_type: u8,
    /// Number of cells
    cell_number: u8
}

/// A tree branch cell
/// 128 bytes per cell
pub struct TreeBranchCell
{
    /// Pointer to the left child
    left_child: u64,
    /// The key
    element_id: u64,
}
pub struct TreeLeafCell
{
    /// The element index
    element_id:     u64,
    /// The total size, including overflow
    size:           u64,
    /// The portion stored on the current page
    initial_size:   u64,
    ///Pointer to the overflow page
    overflow:       u64
}

pub struct OverflowHeader
{
    /// 0 : No overflow, else the next overflow page
    next: u64
}

pub struct Pager
{
    page_size: u64
}