use crate::io::DataBuffer;

use super::{offset::PageOffset, PagerResult, TraitPager, id::PageId};

/// Move a page section, to another page section
pub unsafe fn move_page_section<P: TraitPager>(pager: &mut P, from_page: &PageId, from_offset: &PageOffset, to_page: &PageId, to_offset: &PageOffset, size: usize) -> PagerResult<()> {
    let mut section = DataBuffer::with_size(size as usize);

    unsafe {
        pager.read_from_page(&mut section, from_page, *from_offset)?;
        pager.write_all_to_page(to_page, &mut section, *to_offset)?;
    }

    Ok(())
}