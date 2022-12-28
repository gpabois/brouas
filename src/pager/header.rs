use std::{mem::size_of, io::{Write, BufRead}};

use crate::io::{traits::{OutStream, InStream}, DataStream};

use super::page::{offset::PageOffset, id::PageId, size::PageSize};

#[derive(Default, Copy, Clone)]
pub struct PagerVersion(u64);

impl PagerVersion {
    pub const fn size_of() -> usize {
        size_of::<u64>()
    }
}

impl InStream for PagerVersion {
    fn read_from_stream<R: std::io::BufRead>(&mut self, read: &mut R) -> std::io::Result<()> {
        self.0 = DataStream::<u64>::read(read)?;
        Ok(())
    }
}

impl OutStream for PagerVersion {
    fn write_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        DataStream::<u64>::write(writer, self.0)
    }

    fn write_all_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        DataStream::<u64>::write_all(writer, self.0)
    }
}

#[derive(Default, Clone)]
pub struct PageCount(u64);

impl OutStream for PageCount {
    fn write_to_stream<W: Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        DataStream::<u64>::write(writer, self.0)
    }

    fn write_all_to_stream<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        DataStream::<u64>::write_all(writer, self.0)
    }
}

impl InStream for PageCount {
    fn read_from_stream<R: BufRead>(&mut self, read: &mut R) -> std::io::Result<()> 
    {
        self.0 = DataStream::<u64>::read(read)?;
        Ok(())
    }
}

impl std::ops::AddAssign<u64> for PageCount {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs;
    }
}

impl Into<u64> for PageCount {
    fn into(self) -> u64 {
        self.0
    }
}

impl Into<PageId> for PageCount {
    fn into(self) -> PageId {
        PageId::new(self.into())
    }
}

#[derive(Default)]
pub struct PagerHeader
{
    pub version: PagerVersion,
    /// Size of a page
    pub page_size: PageSize,
    /// Number of pages 
    pub page_count: PageCount,
    /// Offset to pages
    pub page_ptr: PageOffset,
    /// Pointer to the first free page that can be retrieved.
    pub free_head:  Option<PageId>
}

impl PagerHeader {
    pub fn new(page_size: impl Into<PageSize>) -> Self {
        Self { 
            version: PagerVersion(1), 
            page_size: page_size.into(), 
            page_count: PageCount(1),
            page_ptr: Self::size_of().into(), 
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
            self.version.write_to_stream(writer)? +
            self.page_size.write_to_stream(writer)? +
            self.page_count.write_to_stream(writer)? + 
            self.page_ptr.write_to_stream(writer)? +
            self.free_head.write_to_stream(writer)? 
        )
    }

    fn write_all_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.version.write_all_to_stream(writer)?;
        self.page_size.write_all_to_stream(writer)?;
        self.page_count.write_all_to_stream(writer)?;
        self.page_ptr.write_all_to_stream(writer)?;
        self.free_head.write_all_to_stream(writer)
    }
}

impl InStream for PagerHeader {
    fn read_from_stream<R: std::io::BufRead>(&mut self, read: &mut R) -> std::io::Result<()> {
        self.version.read_from_stream(read)?;
        self.page_size.read_from_stream(read)?;
        self.page_count.read_from_stream(read)?;
        self.page_ptr.read_from_stream(read)?;
        self.free_head.read_from_stream(read)?;
        Ok(())
    }
}
