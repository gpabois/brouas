
use std::cmp::max;

use crate::io::{traits::{OutStream, InStream}};

pub mod buffer;
pub mod page;
pub mod id;
pub mod page_type;
pub mod nonce;
pub mod offset;
pub mod traits;
pub mod stream; 
pub mod utils;
pub mod versions;
pub mod error;
pub mod result;
pub mod bptree;

use self::{traits::pager::{Pager as TraitPager, PagerVersion}, result::{PagerResult}, page_type::PageType, id::PageId, page::PageSize, buffer::PagerBuffer, offset::PageOffset, error::PagerError};
use self::traits::stream::PagerStream;

pub struct Pager<V: PagerVersion, S: PagerStream>
{
    page_size: PageSize,
    buffer: PagerBuffer,
    stream: S
}

impl<V: PagerVersion, S: PagerStream> TraitPager for Pager<V, S>
{
    type VERSION = V;

    /// New page
    fn new_page(&mut self, page_type: PageType) -> PagerResult<PageId> 
    {
        let page_id = match V::Allocator::alloc(self)? {
            // We have a free page available !
            Some(free_page_id) => {
                let mut header = V::PageHeader::get(&free_page_id, self)?;
                header.page_type = page_type;
                header.set(self)?;
                free_page_id
            },
            // No free pages
            None => {
                let mut pager_header = V::PagerHeader::get(self)?;

                let page_id: PageId = pager_header.page_count.into();
                pager_header.page_count += 1;
                pager_header.set(self)?;
        
                self.alloc_page_space(&page_id);
        
                let mut header = V::PageHeader::new(page_id);
                header.page_type = page_type;
                header.set(self)?;

                page_id
            }
        };

        Ok(page_id)
    }

    fn open_page(&mut self, page_id: &PageId) -> PagerResult<PageId> 
    {
        // The page is already opened
        if self.buffer.borrow_mut_page(page_id).is_some() {
            Ok(*page_id)
        } else {
            todo!()
        }
    }

    fn close_page(&mut self, page_id: &PageId) -> PagerResult<()> 
    {
        self.buffer.drop_page(page_id);
        Ok(())
    }

    fn flush_page(&mut self, page_id: &PageId) -> PagerResult<()> 
    {
        let page = self.buffer.borrow_mut_page(page_id).ok_or(PagerError::PageNotOpened(*page_id))?;
        self.stream.write_page(page_id, page).map_err(PagerError::from)?;
        Ok(())
    }

    fn flush_modified_pages(&mut self) -> PagerResult<()> {
        todo!()
    }

    fn drop_page(&mut self, page_id: &PageId) -> PagerResult<()> 
    {
        Allocator::free(self, page_id)
    }

    fn assert_page_type(&self, page_id: &PageId, page_type: &PageType) -> PagerResult<()> where Self: Sized {
        let header = V::PageHeader::get::<Self>(page_id, self)?;

        if header.page_type != *page_type {
            return Err(PagerError::WrongPageType { expecting: *page_type, got: header.page_type });
        }

        Ok(())
    }

    unsafe fn write_to_page<D: OutStream, PO: Into<PageOffset>>(&mut self, page_id: &PageId, data: &D, offset: PO) -> PagerResult<usize> 
    {
        let page = self.buffer.borrow_mut_page(page_id).ok_or(PagerError::PageNotOpened(*page_id))?;
        page.write(data, &offset.into())
    }

    unsafe fn write_all_to_page<D: OutStream, PO: Into<PageOffset>>(&mut self, page_id: &PageId, data: &D, offset: PO) -> PagerResult<()> 
    {
        let page = self.buffer.borrow_mut_page(page_id).ok_or(PagerError::PageNotOpened(*page_id))?;
        page.write_all(data, &offset.into())
    }

    /// Read data from page, unsafe because it can read corrupted data if not done correctly.
    unsafe fn read_from_page<D: InStream, PO: Into<PageOffset>>(&self, to: &mut D, page_id: &PageId, offset: PO) -> PagerResult<()> {
        let page = self.buffer.borrow_page(page_id).ok_or(PagerError::PageNotOpened(*page_id))?;
        page.read::<D>(to, &offset.into())
    }

    unsafe fn read_and_instantiate_from_page<D: InStream + Default, PO: Into<PageOffset>>(&self, page_id: &PageId, offset: PO) -> PagerResult<D>
    {
        let mut data: D = Default::default();
        self.read_from_page(&mut data, page_id, offset)?;
        Ok(data)

    }

    fn get_page_size(&self) -> page::PageSize 
    {
        self.page_size
    }

}

impl<V: PagerVersion, S: PagerStream> Pager<V, S>
{
    /// Create a new pager
    /// The page size must be above MIN_PAGE_SIZE.
    pub fn new(stream: S, mut page_size: u64) -> Self 
    {
        page_size = max(V::MIN_PAGE_SIZE, page_size);

        // page_count = 1 because of the root page.
        let mut pager = Self {
            page_size: page_size,
            buffer: PagerBuffer::new(page_size),
            stream: stream
        };

        // init zero page which is the pager header
        pager.alloc_page_space(&PageId::new(0));
        V::PagerHeader::new(page_size).set(&mut pager).unwrap();

        pager
    }

    /// Alloc a page space in the internal buffer
    fn alloc_page_space(&mut self, page_id: &PageId) 
    {
        self.buffer.alloc_page(&page_id);
    }
}

/// Change a page type
pub unsafe fn change_page_type<P>(
    pager: &mut P, 
    page_id: &PageId, 
    new_page_type: PageType) -> PagerResult<()> 
where P: TraitPager
{
    let mut header = P::VERSION::PageHeader::get(page_id, pager)?;
    header.page_type = new_page_type;
    header.set(pager)
}

#[cfg(test)]
mod tests 
{
    use crate::io::DataBuffer;
    use crate::pager::id::PageId;
    use crate::pager::nonce::PageNonce;
    use crate::pager::page_type::PageType;
    use crate::pager::versions::v1::page::header::PageHeader;

    use super::result::PagerResult;
    use super::versions::v1::Pager;
    use super::traits::pager::Pager as TraitPager;

    #[test]
    /// Test the pager.
    pub fn test_pager() -> PagerResult<()> 
    {
        let mut pager = Pager::new(DataBuffer::new(), 1024);
        let expected_page_id = Pager::from(1);

        assert_eq!(expected_page_id, pager.new_page(PageType::BTree)?); 

        // Check the header
        let mut header = PageHeader::get(&expected_page_id, &mut pager)?;

        assert_eq!(header.id,           PageId::from(1));
        assert_ne!(header.nonce,        PageNonce::not_set());
        assert_eq!(header.page_type,    PageType::BTree);

        // Drop the page, should mark it as free.
        pager.drop_page(&expected_page_id)?;

        header = PageHeader::get(&PageId::from(1), &mut pager)?;
        assert_eq!(header.page_type, PageType::Free);

        // Now we create a new page, the pager should be able to recycle the previous dropped page.
        assert_eq!(expected_page_id, pager.new_page(PageType::Raw)?);

        Ok(())
    }
}
