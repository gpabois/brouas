use crate::io::traits::{OutStream, InStream};
use super::{id::PageId, PagerResult, page_type::PageType, page::PageSize, header::PageHeader, PagerError, offset::PageOffset, PagerHeader};

/// Pager stream
pub trait PagerStream
{
    /// Place the cursor to the head of the expected page.
    fn write_page(&mut self, page_id: &PageId, page: &super::page::Page) -> std::io::Result<()>;
    
    /// Read the page content.
    fn read_page(&mut self, page_id: &PageId, page: &mut super::page::Page) -> std::io::Result<()>;
    
    /// Read the pager header from the stream.
    fn read_header(&mut self) -> std::io::Result<PagerHeader>;
}

pub trait Pager 
{
    /// Create a new page.
    fn new_page(&mut self, page_type: PageType) -> PagerResult<PageId>;
    
    /// Open the page from the remote buffer, and store it in the internal buffer.
    fn open_page(&mut self, page_id: &PageId) -> PagerResult<PageId>;
    
    /// Close the page and remove it from the internal buffer, but does not flush it.
    fn close_page(&mut self, page_id: &PageId) -> PagerResult<()>;
    
    /// Flush the page in the remote buffer.
    fn flush_page(&mut self, page_id: &PageId) -> PagerResult<()>;

    /// Flush the pages that have been modified.
    fn flush_modified_pages(&mut self) -> PagerResult<()>;
    
    /// Drop the page, and mark it as free for further reuse.
    fn drop_page(&mut self, page_id: &PageId) -> PagerResult<()>;
    
    /// Assert the page's type behind the id.
    fn assert_page_type(&self, page_id: &PageId, page_type: &PageType) -> PagerResult<()> where Self: Sized {
        let header = PageHeader::get::<Self>(page_id, self)?;

        if header.page_type != *page_type {
            return Err(PagerError::WrongPageType { expecting: *page_type, got: header.page_type });
        }

        Ok(())
    }

    /// Write data to a page.
    /// This method requires the page to be opened.
    unsafe fn write_to_page<D: OutStream, PO: Into<PageOffset>>(&mut self, page_id: &PageId, data: &D, offset: PO) -> PagerResult<usize>;
    
    /// Write data to a page, and ensures that all the data is written.
    /// This method requires the page to be opened.
    unsafe fn write_all_to_page<D: OutStream, PO: Into<PageOffset>>(&mut self, page_id: &PageId, data: &D, offset: PO) -> PagerResult<()>;
    
    /// Read data from a page
    /// This method requires the page to be opened.
    unsafe fn read_from_page<D: InStream, PO: Into<PageOffset>>(&self, to: &mut D, page_id: &PageId, offset: PO) -> PagerResult<()>;
    
    /// Read data from a page, and returns an instance of the read object.
    /// This method requires the page to be opened.
    unsafe fn read_and_instantiate_from_page<D: InStream + Default, PO: Into<PageOffset>>(&self, page_id: &PageId, offset: PO) -> PagerResult<D>
    {
        let mut data: D = Default::default();
        self.read_from_page(&mut data, page_id, offset)?;
        Ok(data)

    }

    fn get_page_size(&self) -> PageSize;
}

pub trait PagerCommandExecutor {
    type Result;
    
    fn execute<P: Pager>(&mut self, pager: &mut P) -> PagerResult<Self::Result>;
}