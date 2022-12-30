use std::fmt::Display;
use std::io::Read;
use crate::io::{Data, DataStream};
use crate::io::traits::{OutStream, InStream};
use crate::pager::overflow::Overflow;
use crate::pager::page::result::PageResult;
use crate::pager::page::{id::PageId, size::BlockSize, offset::PageOffset};
use crate::pager::traits::Pager;

#[derive(Default, Copy, Clone)]
pub struct VarSize(u64);

impl Display for VarSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl VarSize {
    pub const fn size_of() -> usize {
        std::mem::size_of::<u64>()
    }
}

impl From<usize> for VarSize {
    fn from(v: usize) -> Self {
        Self(v as u64)
    }
}

impl Into<usize> for VarSize {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl InStream for VarSize {
    fn read_from_stream<R: Read>(&mut self, read: &mut R) -> std::io::Result<()> {
        self.0 = DataStream::<u64>::read(read)?;
        Ok(())
    }
}

impl OutStream for VarSize {
    fn write_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        DataStream::<u64>::write(writer, self.0)
    }

    fn write_all_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        DataStream::<u64>::write_all(writer, self.0)
    }
}

#[derive(Default)]
pub struct VarHeader {
    size: VarSize,
    overflow: Option<PageId>,
    in_page_size: BlockSize,
    in_page_ptr: PageOffset,
    in_page_capacity: BlockSize
}

impl VarHeader {
    pub const fn size_of() -> usize {
        VarSize::size_of() +
        PageId::size_of() +
        BlockSize::size_of() +
        PageOffset::size_of() +
        BlockSize::size_of()
    }
}

impl InStream for VarHeader {
    fn read_from_stream<R: Read>(&mut self, read: &mut R) -> std::io::Result<()> {
        self.size.read_from_stream(read)?;
        self.overflow.read_from_stream(read)?;
        self.in_page_size.read_from_stream(read)?;
        self.in_page_ptr.read_from_stream(read)?;
        self.in_page_capacity.read_from_stream(read)
    }
}

impl OutStream for VarHeader {
    fn write_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        Ok(
            self.size.write_to_stream(writer)? +
            self.overflow.write_to_stream(writer)? + 
            self.in_page_size.write_to_stream(writer)? +
            self.in_page_ptr.write_to_stream(writer)? +
            self.in_page_capacity.write_to_stream(writer)?
        )
    }

    fn write_all_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.size.write_all_to_stream(writer)?;
        self.overflow.write_all_to_stream(writer)?;
        self.in_page_size.write_all_to_stream(writer)?;
        self.in_page_ptr.write_all_to_stream(writer)?;
        self.in_page_capacity.write_all_to_stream(writer)
    }
}

pub struct Var {
    page_id: PageId,
    base:    PageOffset,
    header:  VarHeader,
    buffer:  Data
}

impl Var 
{
    pub fn new(page_id: PageId, base: PageOffset, capacity: BlockSize) -> Self {
        let mut var = Self {
            page_id: page_id,
            base: base,
            header: Default::default(),
            buffer: Data::new()
        };

        var.header.in_page_ptr = VarHeader::size_of().into();
        let capacity: usize = capacity.into();
        var.header.in_page_capacity = BlockSize::from(capacity - VarHeader::size_of());

        var
    }

    pub fn load<P: Pager>(page_id: PageId, base: PageOffset, pager: &mut P) -> PageResult<Self> {
        let mut var = Self {
            page_id: page_id,
            base: base,
            header: Default::default(),
            buffer: Data::new()
        };

        pager.read_from_page(&mut var.header, &page_id, base)?;
        var.fetch(pager)?;

        Ok(var)
    }

    /// Flush var into the page.
    pub fn flush<P: Pager>(&mut self, pager: &mut P) -> PageResult<()> {
        self.write(pager)?;
        pager.write_all_to_page(&self.page_id, &self.header, self.base)?;
        Ok(())
    }

    /// Fetch var data from the page
    pub fn fetch<P: Pager>(&mut self, pager: &mut P) -> PageResult<()> {
        let mut data = Data::new();

        let mut in_page = Data::with_size(self.header.in_page_size);
        pager.read_from_page(&mut in_page, &self.page_id, self.offset())?;

        data.extend_from_slice(&in_page);
        
        if let Some(overflow_pg_id) = self.header.overflow {
            Overflow::read(pager, &overflow_pg_id, &mut data)?;
        }

        self.buffer = data;

        Ok(())
    }

    pub fn set<O: OutStream>(&mut self, content: &O) -> PageResult<()> {
        let mut data = Data::new();
        content.write_all_to_stream(&mut data)?;
        self.buffer = data;
        Ok(())
    }

    pub fn get<I: InStream>(&self, content: &mut I) -> PageResult<()> {
        content.read_from_stream(&mut self.buffer.get_cursor_read())?;
        Ok(())
    }

    fn offset(&self) -> PageOffset {
        self.base + self.header.in_page_ptr
    }

    fn write<P: Pager>(&mut self, pager: &mut P) -> PageResult<()> {
        let mut in_page = Data::with_size(self.header.in_page_capacity);

        let mut cursor = self.buffer.get_cursor_read();
        self.header.in_page_size = cursor.read(&mut in_page)?.into();
        self.header.size = self.buffer.len().into();
        
        pager.write_all_to_page(&self.page_id, &in_page, self.offset())?;
        self.header.overflow = Overflow::write(pager, &mut cursor, self.header.overflow)?;
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{pager::page::{result::PageResult, page_type::PageType, offset::PageOffset}, fixtures, io::Data};
    use crate::pager::traits::Pager;

    use super::{Var};

    #[test]
    fn test_var() -> PageResult<()> {
        let data = fixtures::random_data(10000);
        let mut stored_data = Data::with_size(10000usize);
        
        let mut pager = fixtures::pager_fixture(4000usize);
        let pg_id = pager.new_page(PageType::Raw)?;
        
        let base = PageOffset::from(0u64);
        let capacity = pager.get_page_capacity(&pg_id)?;
        
        // Create a var data
        {
            let mut var = Var::new(pg_id, base, capacity);
            // Set the var content.
            var.set(&data)?;
            var.flush(&mut pager)?;
        }

        // Load the var data and get its content.
        {
            let var = Var::load(pg_id, base, &mut pager)?;
            var.get(&mut stored_data)?;
            assert_eq!(data, stored_data);
        }

        Ok(())
    }
}