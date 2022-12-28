
use std::{cmp::max, io::{BufRead, Write, Seek, SeekFrom}};
use crate::io::{traits::{OutStream, InStream}};
use self::{page::{id::PageId, result::PageResult, Page, page_type::PageType, error::PageError, offset::PageOffset, MIN_PAGE_SIZE, size::PageSize}, buffer::PagerBuffer, header::PagerHeader, allocator::Allocator};
pub use self::traits::Pager as TraitPager;

pub mod buffer;
pub mod page;
pub mod overflow; 
pub mod allocator;
pub mod traits;
pub mod utils;
pub mod header;
// pub mod bptree;


pub struct Pager<S: BufRead + Write + Seek>
{
    header: PagerHeader,
    buffer: PagerBuffer,
    stream: S
}

impl<S: BufRead + Write + Seek> TraitPager for Pager<S>
{
    /// New page
    fn new_page(&mut self, page_type: PageType) -> PageResult<PageId> 
    {
        let page_id = match Allocator::alloc(self)? {
            // We have a free page available ! We recycle it
            Some(free_page_id) => {
                unsafe {
                    self.change_page_type(&free_page_id, page_type)?;
                }
                free_page_id
            },
            // No free pages
            None => {
                let page_id = self.header.page_count.clone().into();
                let page = Page::new(page_id, self.get_page_size(), page_type);
                self.buffer.add(page);
                self.header.page_count += 1;    

                page_id
            }
        };

        Ok(page_id)
    }

    fn open_page(&mut self, page_id: &PageId) -> PageResult<PageId> 
    {
        // The page is already opened
        if self.buffer.borrow_mut_page(page_id).is_some() {
            Ok(*page_id)
        } else {
            // Seek page from the stream
            todo!()
        }
    }

    fn close_page(&mut self, page_id: &PageId) -> PageResult<()> 
    {
        self.buffer.drop_page(page_id);
        Ok(())
    }

    fn flush_page(&mut self, page_id: &PageId) -> PageResult<()> 
    {
        let size: u64 = self.get_page_size().into();
        let page = self.buffer.borrow_mut_page(page_id).ok_or(PageError::PageNotOpened(*page_id))?;
        let mut addr: u64 = self.header.page_ptr.into();
        
        let id: u64 = (*page_id).into();

        addr = addr.wrapping_add(id.wrapping_mul(size));

        self.stream.seek(SeekFrom::Start(addr)).map_err(PageError::from)?;
        self.stream.write_all(&page).map_err(PageError::from)
    }

    fn flush(&mut self) -> PageResult<()> {
        self.stream.seek(SeekFrom::Start(0)).map_err(PageError::from)?;
        self.header.write_all_to_stream(&mut self.stream).map_err(PageError::from)
    }

    fn drop_page(&mut self, page_id: &PageId) -> PageResult<()> 
    {
        Allocator::free(self, page_id)
    }

    fn assert_page_type(&self, page_id: &PageId, page_type: &PageType) -> PageResult<()> where Self: Sized {
        let pg = self.buffer.borrow_page(page_id).ok_or(PageError::PageNotOpened(*page_id))?;

        if pg.get_type() != *page_type {
            return Err(PageError::WrongPageType { expecting: *page_type, got: pg.get_type() });
        }

        Ok(())
    }

    fn get_body_ptr(&self, page_id: &PageId) -> PageResult<page::offset::PageOffset> {
        let pg = self.buffer.borrow_page(page_id).ok_or(PageError::PageNotOpened(*page_id))?;
        Ok(pg.get_body_ptr())
    }

    // Write data to page body.
    fn write_to_page<D: OutStream, PO: Into<PageOffset>>(&mut self, page_id: &PageId, data: &D, offset: PO) -> PageResult<usize> 
    {
        let page = self.buffer.borrow_mut_page(page_id).ok_or(PageError::PageNotOpened(*page_id))?;
        let offset = page.get_body_ptr() + offset.into();
        unsafe {
            page.write(data, offset).map_err(PageError::from)
        }
    }

    // Write data to page body.
    fn write_all_to_page<D: OutStream, PO: Into<PageOffset>>(&mut self, page_id: &PageId, data: &D, offset: PO) -> PageResult<()> 
    {
        let page = self.buffer.borrow_mut_page(page_id).ok_or(PageError::PageNotOpened(*page_id))?;
        let offset = page.get_body_ptr() + offset.into();
        unsafe {
            page.write_all(data, offset).map_err(PageError::from)
        }
    }

    /// Read data from page body.
    fn read_from_page<D: InStream, PO: Into<PageOffset>>(&self, to: &mut D, page_id: &PageId, offset: PO) -> PageResult<()> {
        let page = self.buffer.borrow_page(page_id).ok_or(PageError::PageNotOpened(*page_id))?;
        let offset = page.get_body_ptr() + offset.into();
        unsafe {
            page.read::<D>(to, offset).map_err(PageError::from)
        }
    }

    fn read_and_instantiate_from_page<D: InStream + Default, PO: Into<PageOffset>>(&self, page_id: &PageId, offset: PO) -> PageResult<D>
    {
        let mut data: D = Default::default();
        self.read_from_page(&mut data, page_id, offset)?;
        Ok(data)

    }

    fn get_page_size(&self) -> PageSize 
    {
        self.header.page_size
    }

    fn get_freelist_head(&self) -> Option<PageId> {
        self.header.free_head
    }

    fn set_freelist_head(&mut self, new_head: Option<PageId>) {
        self.header.free_head = new_head;
    }

    fn flush_modified_pages(&mut self) -> PageResult<()> {
        todo!()
    }

    unsafe fn change_page_type(&mut self, page_id: &PageId, page_type: PageType) -> PageResult<()> {
        let page = self.buffer.borrow_mut_page(page_id).ok_or(PageError::PageNotOpened(*page_id))?;
        page.set_type(page_type);
        Ok(())
    }

    fn get_page_metadata(&self, page_id: &PageId) -> PageResult<page::metadata::PageMetadata> {
        let page = self.buffer.borrow_page(page_id).ok_or(PageError::PageNotOpened(*page_id))?;
        Ok(page.get_metadata())
    }

}

impl<Stream: BufRead + Write + Seek> Pager<Stream>
{
    /// Create a new pager
    /// The page size must be above MIN_PAGE_SIZE.
    pub fn new(stream: Stream, page_size: impl Into<PageSize>) -> Self 
    {
        let page_size = max(MIN_PAGE_SIZE, page_size.into().into());

        // page_count = 1 because of the root page.
        let pager = Self {
            header: PagerHeader::new(page_size),
            buffer: PagerBuffer::new(),
            stream: stream
        };

        pager
    }
}

#[cfg(test)]
mod tests 
{
    use crate::{pager::{page::{id::PageId, page_type::PageType, nonce::PageNonce}}, fixtures::pager_fixture};

    use super::page::result::PageResult;
    use super::TraitPager;

    #[test]
    /// Test the pager.
    pub fn test_pager() -> PageResult<()> 
    {
        let mut pager = pager_fixture(1024u64);
        let expected_page_id = PageId::from(1);

        assert_eq!(expected_page_id, pager.new_page(PageType::BTree)?); 
        
        // Check the page metadata
        let mut meta = pager.get_page_metadata(&expected_page_id)?;

        assert_eq!(meta.id,           PageId::from(1));
        assert_ne!(meta.nonce,        PageNonce::not_set());
        assert_eq!(meta.page_type,    PageType::BTree);

        // Drop the page, should mark it as free.
        pager.drop_page(&expected_page_id)?;

        meta = pager.get_page_metadata(&expected_page_id)?;
        assert_eq!(meta.page_type, PageType::Free);

        // Now we create a new page, the pager should be able to recycle the previous dropped page.
        assert_eq!(expected_page_id, pager.new_page(PageType::Raw)?);

        Ok(())
    }
}
