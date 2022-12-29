use std::io::Read;

use crate::io::Data;
use crate::io::traits::{OutStream, InStream};
use crate::pager::overflow::Overflow;
use crate::pager::page::result::PageResult;
use crate::pager::page::{id::PageId, size::BlockSize, offset::PageOffset};
use crate::pager::traits::Pager;

#[derive(Default, Copy, Clone)]
pub struct VarSize(u64);

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

#[derive(Default)]
pub struct VarHeader {
    size: VarSize,
    overflow: Option<PageId>,
    in_page_size: BlockSize,
    in_page_ptr: PageOffset,
    in_page_capacity: BlockSize
}

pub struct Var {
    page_id: PageId,
    base:    PageOffset,
    header:  VarHeader
}

impl Var 
{
    pub fn flush<P: Pager>(&self) {

    }

    pub fn write<P: Pager, O: OutStream>(&mut self, pager: &mut P, content: &O) -> PageResult<()> {
        // Write the content into a data buffer
        let mut data = Data::new();
        content.write_all_to_stream(&mut data)?;
        
        let mut in_page = Data::with_size(self.header.in_page_capacity);

        let mut cursor = data.get_cursor_read();
        self.header.in_page_size = cursor.read(&mut in_page)?.into();
        self.header.size = data.len().into();

        let offset = self.base + self.header.in_page_ptr;

        pager.write_all_to_page(&self.page_id, &in_page, offset)?;
        Overflow::write(pager, &mut cursor, self.header.overflow)?;

        Ok(())
    }

    pub fn read<P: Pager, I: InStream>(&self, pager: &mut P, to: &mut I) -> PageResult<()> {
        let mut data = Data::new();

        let mut in_page = Data::with_size(self.header.in_page_size);
        let offset = self.base + self.header.in_page_ptr;
        pager.read_from_page(&mut in_page, &self.page_id, offset)?;
        data.extend_from_slice(&in_page);

        if let Some(overflow_pg_id) = self.header.overflow {
            Overflow::read(pager, &overflow_pg_id, &mut data)?;
        }

        to.read_from_stream(&mut data)?;

        Ok(())
    }
}