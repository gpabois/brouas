use crate::{io::traits::{OutStream, InStream}, pager::{offset::PageOffset, id::PageId, PagerResult, page::PageSize, page_type::PageType, error::PagerError}};

pub trait PagerVersion 
{
    // Page management unit, to reuse freed pages.
    type Allocator;
    // The type of pager header.
    type PagerHeader;
    // the type of page header.
    type PageHeader;
    // Minimum size of page
    const MIN_PAGE_SIZE: u64;
}

pub trait Pager 
{   
    type VERSION: PagerVersion;

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
        let header = Self::VERSION::PageHeader::get::<Self>(page_id, self)?;

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

