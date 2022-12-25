use crate::pager::{id::PageId, page::Page};

pub trait PagerStream
{
    /// Place the cursor to the head of the expected page.
    fn write_page(&mut self, page_id: &PageId, page: &Page) -> std::io::Result<()>;
    
    /// Read the page content.
    fn read_page(&mut self, page_id: &PageId, page: &mut Page) -> std::io::Result<()>;
    
    /// Read the pager version.
    fn read_version(&mut self) -> std::io::Result<u64>;
}
