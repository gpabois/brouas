use std::io::{BufRead, BufWriter, Write, BufReader, Read, Seek, SeekFrom};

use super::page_id::PageId;
use super::page_nonce::PageNonce;
use super::page_type::PageType;

/// Header of page
/// Size: 88 bytes
pub struct PageHeader
{
    /// Number of the page.
    id: PageId,
    /// Nonce, in case of conflicted pages.
    nonce: PageNonce,
    /// Type of page :
    /// + 0 = Collection Tree ;
    /// + 1 = B+ Tree ;
    /// + 2 = Overflow page.
    page_type: PageType
}

impl PageHeader {
    /// Size of the page header.
    pub fn raw_size_of() -> u64 { PageId::raw_size_of() + PageNonce::raw_size_of() + PageType::raw_size_of() }

    pub fn seek_end<S: Seek>(s: &mut S) -> std::io::Result<u64>
    {
        s.seek(SeekFrom::Start(Self::raw_size_of()))
    }

    pub fn seek_page_type<S: Seek + Read>(b: &mut BufReader<S>) -> std::io::Result<PageType>
    {
        b.seek(SeekFrom::Start(0));
        let page_type_pos = (PageId::raw_size_of() + PageNonce::raw_size_of()) as i64;
        b.seek_relative(page_type_pos)?;
        PageType::read_from_buffer(b)
    }

    pub fn write_to_buffer<W: Write>(&self, b: &mut BufWriter<W>) -> std::io::Result<usize>
    {
        Ok(
            self.id.write_to_buffer(b)? + 
            self.nonce.write_to_buffer(b)? + 
            self.page_type.write_to_buffer(b)?
        )
    }

    pub fn read_from_buffer<B: BufRead>(b: &mut B) -> std::io::Result<Self>
    {
        let id = PageId::read_from_buffer(b)?;
        let nonce = PageNonce::read_from_buffer(b)?;
        let page_type = PageType::read_from_buffer(b)?;

        Ok(Self {
            id: id,
            nonce: nonce,
            page_type: page_type
        })
    }
}