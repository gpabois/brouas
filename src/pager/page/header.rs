use std::io::{Read, Write};

use crate::io::traits::{OutStream, InStream};

use super::offset::PageOffset;
use super::id::PageId;
use super::nonce::PageNonce;
use super::page_type::PageType;

/// Header of page
/// Size: 88 bytes
#[derive(Default, Clone)]
pub struct PageHeader
{
    /// Number of the page.
    pub id: PageId,
    /// Nonce, in case of conflicted pages.
    pub nonce: PageNonce,
    /// Pointer to the body base of the page
    pub body_ptr: PageOffset,
    /// Type of page 
    pub page_type: PageType,
    /// Parent page
    pub parent_id: Option<PageId>
}

const PAGE_HEADER_SIZE: usize = PageHeader::size_of();

impl OutStream for PageHeader {
    fn write_to_stream<W: Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        Ok(
            self.id.write_to_stream(writer)? + 
            self.nonce.write_to_stream(writer)? + 
            self.body_ptr.write_to_stream(writer)? +
            self.page_type.write_to_stream(writer)? +
            self.parent_id.write_to_stream(writer)?
        )
    }

    fn write_all_to_stream<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.id.write_all_to_stream(writer)?;
        self.nonce.write_all_to_stream(writer)?;
        self.body_ptr.write_to_stream(writer)?;
        self.page_type.write_all_to_stream(writer)?;
        self.parent_id.write_all_to_stream(writer)?;
        Ok(())
    }
}

impl InStream for PageHeader 
{
    fn read_from_stream<R: Read>(&mut self, reader: &mut R) -> std::io::Result<()> {
        self.id.read_from_stream(reader)?;
        self.nonce.read_from_stream(reader)?;
        self.body_ptr.read_from_stream(reader)?;
        self.page_type.read_from_stream(reader)?;
        self.parent_id.read_from_stream(reader)?;
        Ok(())
    }
}

impl PageHeader 
{
    /// Size of the page header.
    pub const fn size_of() -> usize { 
        PageId::size_of() + 
        PageNonce::size_of() +
        PageOffset::size_of()  +
        PageType::size_of() + 
        PageId::size_of() 
    }

    pub fn new(page_id: PageId) -> Self {
        Self { 
            id: page_id, 
            nonce: PageNonce::new(), 
            page_type: PageType::Unitialised, 
            body_ptr: Self::size_of().into(),
            parent_id: None
        }
    }
}
