use crate::io::DataBuffer;

use super::page::{result::PageResult, offset::PageOffset, id::PageId};
use super::traits::Pager;

/// Move a page section, to another page section
pub unsafe fn move_page_section<P: Pager>(pager: &mut P, from_page: &PageId, from_offset: &PageOffset, to_page: &PageId, to_offset: &PageOffset, size: usize) -> PageResult<()> {
    let mut section = DataBuffer::with_size(size as usize);

    pager.read_from_page(&mut section, from_page, *from_offset)?;
    pager.write_all_to_page(to_page, &mut section, *to_offset)?;

    Ok(())
}