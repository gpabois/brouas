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

pub enum PageType
{
    Collection,
    BTree,
    Overflow
}

impl Into<u8> for &PageType
{
    fn into(self) -> u8 {
        match self {
            PageType::Collection => 0,
            PageType::BTree => 1,
            PageType::Overflow => 2
        }
    }
}
impl From<u8> for PageType
{
    fn from(value: u8) -> Self {
        match value {
            0 => PageType::Collection,
            1 => PageType::BTree,
            2 => PageType::Overflow,
            _ => panic!("unknown type of page")
        }
    }
}

/// Header of page
/// Size: 88 bytes
pub struct PageHeader
{
    /// Number of the page
    id:         PageId,
    /// Unique number, in case of conflicted pages
    nonce:      u16,
    /// Type of page, 1 = Collection Tree, 0 = B+ Tree, 1 = Overflow page
    page_type:  PageType
}

pub enum TreeNodeType
{
    Leaf,
    Branch
}

impl Into<u8> for &TreeNodeType
{
    fn into(self) -> u8 {
        match self {
            TreeNodeType::Branch => 0,
            TreeNodeType::Leaf => 1
        }
    }
}

impl From<u8> for TreeNodeType
{
    fn from(value: u8) -> Self {
        match value {
            0 => TreeNodeType::Branch,
            1 => TreeNodeType::Leaf,
            _ => panic!("unknown type of b+ tree node")
        }
    }
}


pub struct TreePageHeader
{
    /// Type of node: 1 = Leaf, 0 = Branch
    node_type: TreeNodeType,
    /// Number of cells
    cell_number: u8
}

/// A tree branch cell
/// Size: 128 bytes per cell
pub struct TreeBranchCell
{
    /// Pointer to the left child
    left_child: u64,
    /// The key
    element_id: u64,
}

/// A tree leaf cell
/// Header: 256 bytes per cell
/// Payload: Page size - 256 - Page header size
/// If > payload: throw the remaining into overflow pages
pub struct TreeLeafCell
{
    /// The element index
    element_id: u64,
    /// The total size, including overflow
    size: u64,
    /// The portion stored on the current page
    initial_size:   u64,
    ///Pointer to the overflow page
    overflow: PageId
}

/// Header of an overflow page
/// Size: 64 bytes
pub struct OverflowHeader
{
    /// 0 : No overflow, else the next overflow page
    next: u64
}

pub struct PagerHeader
{
    page_size: u64,
    page_count: u64
}

pub struct Pager
{
    header: PagerHeader
}