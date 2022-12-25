use crate::{io::{DataStream, traits::{OutStream, InStream}}, pager::{page::{PageSize}, id::PageId, traits::pager::Pager, PagerResult}};

pub const PAGER_HEADER_SIZE: u64 = PagerHeader::size_of();
const PAGER_PAGE_INDEX: PageId = 0.into();

#[derive(Default)]
pub struct PagerHeader
{
    /// Should alway be first
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

impl PagerHeader 
{
    pub fn set<P: Pager>(&self, pager: &mut P) -> PagerResult<()> 
    {
        unsafe 
        {
            pager.write_all_to_page(&PAGER_PAGE_INDEX, self, 0u64)
        }
    }

    pub fn get<P: Pager>(pager: &P) -> PagerResult<Self> 
    {
        unsafe 
        {
            pager.read_and_instantiate_from_page::<Self, _>(&PAGER_PAGE_INDEX, 0u64)
        }
    }
}