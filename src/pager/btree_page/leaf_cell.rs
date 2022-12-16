use crate::pager::page_id::PageId;

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
