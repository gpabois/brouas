
use std::cmp::max;

use crate::io::{traits::{OutStream, InStream}, DataStream};

use self::{header::{PageHeader, PAGE_HEADER_SIZE}, id::PageId, page_type::PageType, offset::{PageOffset}, page::{PageSize}, buffer::PagerBuffer, allocator::Allocator, traits::PagerStream};
pub use self::traits::Pager as TraitPager;

pub mod buffer;
pub mod page;
pub mod id;
pub mod page_type;
pub mod header;
pub mod nonce;
pub mod offset;
pub mod overflow; 
pub mod btree;
pub mod allocator;
pub mod traits;
pub mod stream; 

#[derive(Debug)]
pub enum PagerError
{
    IOError(std::io::Error),
    WrongPageType{expecting: PageType, got: PageType},
    PageNotOpened(PageId),
    PageOverflow,
    PageFull(PageId)
}

impl From<std::io::Error> for PagerError {
    fn from(err: std::io::Error) -> Self {
        Self::IOError(err)
    }
}

pub type PagerResult<T> = std::result::Result<T, PagerError>;

#[derive(Default)]
pub struct PagerHeader
{
    pub version:    u64,
    /// Size of a page
    pub page_size:  u64,
    /// Number of pages 
    pub page_count: u64,
    /// Pointer to the first free page that can be retrieved.
    pub free_head:  Option<PageId>
}

impl PagerHeader {
    fn new(page_size: PageSize) -> Self {
        Self { 
            version: 1, 
            page_size: page_size, 
            page_count: 1, 
            free_head: Default::default() 
        }
    }

    pub const fn size_of() -> u64 {
        4 * 8
    }
}

const PAGER_HEADER_SIZE: u64 = PagerHeader::size_of();

impl OutStream for PagerHeader {
    fn write_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        Ok( 
            DataStream::<u64>::write(writer, self.version)? +
            DataStream::<u64>::write(writer, self.page_size)? +
            DataStream::<u64>::write(writer, self.page_count)? +
            self.free_head.write_to_stream(writer)? 
        )
    }

    fn write_all_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        DataStream::<u64>::write_all(writer, self.version)?;
        DataStream::<u64>::write_all(writer, self.page_size)?;
        DataStream::<u64>::write_all(writer, self.page_count)?;
        self.free_head.write_all_to_stream(writer)
    }
}

impl InStream for PagerHeader {
    fn read_from_stream<R: std::io::BufRead>(&mut self, read: &mut R) -> std::io::Result<()> {
        self.version = DataStream::<u64>::read(read)?;
        self.page_size = DataStream::<u64>::read(read)?;
        self.page_count = DataStream::<u64>::read(read)?;
        self.free_head.read_from_stream(read)?;
        Ok(())
    }
}

const PAGER_PAGE_INDEX: PageId = PageId::new(0);

impl PagerHeader {
    pub fn set<P: TraitPager>(&self, pager: &mut P) -> PagerResult<()> 
    {
        unsafe 
        {
            pager.write_all_to_page(&PAGER_PAGE_INDEX, self, 0u64)
        }
    }

    pub fn get<P: TraitPager>(pager: &P) -> PagerResult<Self> 
    {
        unsafe 
        {
            pager.read_and_instantiate_from_page::<Self, _>(&PAGER_PAGE_INDEX, 0u64)
        }
    }
}

pub struct Pager<Stream: PagerStream>
{
    page_size: PageSize,
    buffer: PagerBuffer,
    stream: Stream
}

impl<Stream: PagerStream> TraitPager for Pager<Stream>
{
    /// New page
    fn new_page(&mut self, page_type: PageType) -> PagerResult<PageId> 
    {
        let page_id = match Allocator::alloc(self)? {
            // We have a free page available !
            Some(free_page_id) => {
                let mut header = PageHeader::get(&free_page_id, self)?;
                header.page_type = page_type;
                header.set(self)?;
                free_page_id
            },
            // No free pages
            None => {
                let mut pager_header = PagerHeader::get(self)?;

                let page_id: PageId = pager_header.page_count.into();
                pager_header.page_count += 1;
                pager_header.set(self)?;
        
                self.alloc_page_space(&page_id);
        
                let mut header = PageHeader::new(page_id);
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
        let header = PageHeader::get::<Self>(page_id, self)?;

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

const MIN_PAGE_SIZE: u64 = const_utils::u64::max(PAGE_HEADER_SIZE, PAGER_HEADER_SIZE);

impl<Stream: PagerStream> Pager<Stream>
{
    /// Create a new pager
    /// The page size must be above MIN_PAGE_SIZE.
    pub fn new(stream: Stream, mut page_size: u64) -> Self 
    {
        page_size = max(MIN_PAGE_SIZE, page_size);

        // page_count = 1 because of the root page.
        let mut pager = Self {
            page_size: page_size,
            buffer: PagerBuffer::new(page_size),
            stream: stream
        };

        // init zero page which is the pager header
        pager.alloc_page_space(&PageId::new(0));
        PagerHeader::new(page_size).set(&mut pager).unwrap();

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
    let mut header = PageHeader::get(page_id, pager)?;
    header.page_type = new_page_type;
    header.set(pager)
}

#[cfg(test)]
mod tests 
{

    use crate::{pager::{id::PageId, page_type::PageType, header::PageHeader, nonce::PageNonce}, io::DataBuffer};
    use crate::pager::traits::Pager as TraitPager;

    use super::{PagerResult, Pager};

    #[test]
    /// Test the pager.
    pub fn test_pager() -> PagerResult<()> 
    {
        let mut pager = Pager::new(DataBuffer::new(), 1024);
        let expected_page_id = PageId::from(1);

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
