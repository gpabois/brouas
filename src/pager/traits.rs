use crate::io::traits::{OutStream, InStream};

use super::page::{page_type::PageType, id::PageId, result::PageResult, offset::PageOffset, metadata::PageMetadata, size::PageSize};

pub trait Pager 
{
    /// Create a new page.
    fn new_page(&mut self, page_type: PageType) -> PageResult<PageId>;
    
    /// Open the page from the remote buffer, and store it in the internal buffer.
    fn open_page(&mut self, page_id: &PageId) -> PageResult<PageId>;
    
    /// Close the page and remove it from the internal buffer, but does not flush it.
    fn close_page(&mut self, page_id: &PageId) -> PageResult<()>;

    fn close_all(&mut self) -> PageResult<()>;
    
    /// Flush the page in the remote buffer.
    fn flush_page(&mut self, page_id: &PageId) -> PageResult<()>;

    /// Flush all the new/updated pages
    fn flush(&mut self) -> PageResult<()>;

    /// Drop the page, and mark it as free for further reuse.
    fn drop_page(&mut self, page_id: &PageId) -> PageResult<()>;
    
    /// Assert the page's type behind the id.
    fn assert_page_type(&self, page_id: &PageId, page_type: &PageType) -> PageResult<()>;

    /// The the pointer to the body of the page
    fn get_body_ptr(&self, page_id: &PageId) -> PageResult<PageOffset>;

    /// Write data to a page.
    /// This method requires the page to be opened.
    fn write_to_page<D: OutStream, PO: Into<PageOffset>>(&mut self, page_id: &PageId, data: &D, offset: PO) -> PageResult<usize>;
    
    /// Write data to a page, and ensures that all the data is written.
    /// This method requires the page to be opened.
    fn write_all_to_page<D: OutStream, PO: Into<PageOffset>>(&mut self, page_id: &PageId, data: &D, offset: PO) -> PageResult<()>;
    
    /// Read data from a page
    /// This method requires the page to be opened.
    fn read_from_page<D: InStream, PO: Into<PageOffset>>(&self, to: &mut D, page_id: &PageId, offset: PO) -> PageResult<()>;
    
    /// Read data from a page, and returns an instance of the read object.
    /// This method requires the page to be opened.
    fn read_and_instantiate_from_page<D: InStream + Default, PO: Into<PageOffset>>(&self, page_id: &PageId, offset: PO) -> PageResult<D>
    {
        let mut data: D = Default::default();
        self.read_from_page(&mut data, page_id, offset)?;
        Ok(data)

    }

    unsafe fn change_page_type(&mut self, page_id: &PageId, page_type: PageType) -> PageResult<()>;
    
    fn get_page_metadata(&self, page_id: &PageId) -> PageResult<PageMetadata>;
    fn get_page_size(&self) -> PageSize;
    fn get_freelist_head(&self) -> Option<PageId>;
    fn set_freelist_head(&mut self, newt_head: Option<PageId>);
}
