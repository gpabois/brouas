use std::io::{BufRead, Write};

use crate::io::traits::{OutStream, InStream};

use super::offset::PAGE_HEADER_OFFSET;
use super::{PagerResult, TraitPager};

use super::id::PageId;
use super::nonce::PageNonce;
use super::page_type::PageType;

/// Header of page
/// Size: 88 bytes
#[derive(Default)]
pub struct PageHeader
{
    /// Number of the page.
    pub id: PageId,
    /// Nonce, in case of conflicted pages.
    pub nonce: PageNonce,
    /// Type of page :
    /// + 0 = Unitialized ; 
    /// + 1 = Collection Tree ;
    /// + 2 = B+ Tree ;
    /// + 3 = Overflow page.
    pub page_type: PageType,
    /// Parent page
    pub parent_id: Option<PageId>
}
impl OutStream for PageHeader {
    fn write_to_stream<W: Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        Ok(
            self.id.write_to_stream(writer)? + 
            self.nonce.write_to_stream(writer)? + 
            self.page_type.write_to_stream(writer)? +
            self.parent_id.write_to_stream(writer)?
        )
    }

    fn write_all_to_stream<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.id.write_all_to_stream(writer)?;
        self.nonce.write_all_to_stream(writer)?;
        self.page_type.write_all_to_stream(writer)?;
        self.parent_id.write_all_to_stream(writer)?;
        Ok(())
    }
}

impl InStream for PageHeader 
{
    fn read_from_stream<R: BufRead>(&mut self, reader: &mut R) -> std::io::Result<()> {
        self.id.read_from_stream(reader)?;
        self.nonce.read_from_stream(reader)?;
        self.page_type.read_from_stream(reader)?;
        self.parent_id.read_from_stream(reader)?;
        Ok(())
    }
}

impl PageHeader 
{
    /// Size of the page header.
    pub const fn size_of() -> u64 { PageId::size_of() + PageNonce::size_of() + PageType::size_of() + PageId::size_of() }

    pub fn new(page_id: PageId) -> Self {
        Self { id: page_id, nonce: PageNonce::new(), page_type: PageType::Unitialised, parent_id: None}
    }

    pub fn set<P: TraitPager>(&self, pager: &mut P) -> PagerResult<()> {
        unsafe {
            pager.write_all_to_page(&self.id, self, PAGE_HEADER_OFFSET)
        }
    }
    pub fn get<P: TraitPager>(page_id: &PageId, pager: &P) -> PagerResult<Self> {
        unsafe {
            pager.read_and_instantiate_from_page::<Self, _>(page_id, PAGE_HEADER_OFFSET)
        }
    }
}

pub const PAGE_HEADER_SIZE: u64 = PageHeader::size_of();