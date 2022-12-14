use std::io::{Seek, Read, Write};

use crate::io::{Data, DataRef};
use crate::io::traits::{OutStream, InStream};

use super::page::error::PageError;
use super::page::id::PageId;
use super::page::offset::PageOffset;
use super::page::page_type::PageType;
use super::page::result::PageResult;
use super::page::size::BlockSize;
use super::traits::Pager;

/// Header of an overflow page
#[derive(Default)]
pub struct OverflowHeader {
    pub next: Option<PageId>,
    pub in_page_size: BlockSize,
    pub in_page_ptr: PageOffset
}

impl OutStream for OverflowHeader {
    fn write_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        Ok(
            self.next.write_to_stream(writer)? + 
            self.in_page_size.write_to_stream(writer)? + 
            self.in_page_ptr.write_to_stream(writer)?
        )
    }

    fn write_all_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.next.write_all_to_stream(writer)?;
        self.in_page_size.write_all_to_stream(writer)?;
        self.in_page_ptr.write_all_to_stream(writer)?;
        Ok(())
    }
}

impl InStream for OverflowHeader {
    fn read_from_stream<R: std::io::Read>(&mut self, reader: &mut R) -> std::io::Result<()> {
        self.next.read_from_stream(reader)?;
        self.in_page_size.read_from_stream(reader)?;
        self.in_page_ptr.read_from_stream(reader)?;
        Ok(())
    }
}

impl OverflowHeader {
    pub const fn size_of() -> usize { PageId::size_of() + BlockSize::size_of() + PageOffset::size_of() }
}

/// Overflow page wrapper
pub struct OverflowPage {
    page_id:        PageId,
    base:           PageOffset,
    max_body_size:  BlockSize,
    header:         OverflowHeader
}

impl OverflowPage {
    pub fn new<P: Pager>(pager: &mut P) -> PageResult<Self> {
        let page_id = pager.new_page(PageType::Overflow)?;
        let base: PageOffset = 0u64.into();
        let body_ptr = pager.get_body_ptr(&page_id)?;

        let mut header = OverflowHeader::default();
        header.in_page_ptr = base + OverflowHeader::size_of();

        Ok(Self {
            page_id: page_id,
            base: base,
            max_body_size: pager.get_page_size() - (header.in_page_ptr + body_ptr) ,
            header: header
        })
    }

    pub fn get<P: Pager>(page_id: &PageId, pager: &mut P) -> PageResult<Self> {
        pager.open_page(page_id)?;
        pager.assert_page_type(page_id, &PageType::Overflow)?;
        
        let base = 0u64.into();
        let body_ptr = pager.get_body_ptr(&page_id)?;
        let header = pager.read_and_instantiate_from_page::<OverflowHeader, _>(page_id, base)?;
        
        Ok(Self {
            page_id: *page_id,
            base: base,
            max_body_size: pager.get_page_size() - (header.in_page_ptr + body_ptr),
            header: header
        })
    }

    /// Read data from the page.
    pub fn read<P: Pager>(&self, pager: &mut P) -> PageResult<Data> {
        let mut data = Data::with_size(self.header.in_page_size);
        pager.read_from_page(&mut data, &self.page_id, self.header.in_page_ptr + self.base)?;
        Ok(data)
    }

    /// Write data directly into the page.
    pub fn write<P: Pager, R: Read + Seek>(&mut self, pager: &mut P, data: &mut R) -> PageResult<usize> {
        let mut buf = Data::with_size(self.max_body_size);
        let written = data.read(&mut buf)?;
        pager.write_all_to_page(
            &self.page_id, 
            &DataRef::new(&buf[..written]), 
            self.header.in_page_ptr + self.base
        )?;
        self.header.in_page_size = written.into();
        Ok(written)
    }

    /// Flush the metadata in the page.
    pub fn flush<P: Pager>(&mut self, pager: &mut P) -> PageResult<()> {
        pager.write_all_to_page(&self.page_id, &self.header, self.base)
    }

    pub fn set_next(&mut self, next: Option<PageId>) {
        self.header.next = next;
    }

    pub fn get_next(&self) -> Option<PageId>  {
        self.header.next
    }
}

/// A tool to transfer overflowed data into dedicated pages.
pub struct Overflow {
    /// Page size
    pub page_size: u64
}

impl Overflow {
    /// Write data into overflow pages.
    pub fn write<P: Pager, R: Read + Seek>(pager: &mut P, data: &mut R, head: Option<PageId>) -> PageResult<Option<PageId>>
    {
        let mut target: Option<PageId> = head;
        let mut prev: Option<PageId> = None;
        let mut head: Option<PageId> = None;

        // Write data chunk into an overflow page.
        while let Some(mut target_pg) = Self::write_overflow(pager, data, target, prev)? {
            target = target_pg.get_next();
            if head.is_none() { head = Some(target_pg.page_id); }
            prev = Some(target_pg.page_id);

            target_pg.flush(pager)?;
        }

        Ok(head)
    }
    
    fn write_overflow<P: Pager, R: Read + Seek>(pager: &mut P, data: &mut R, target_page_id: Option<PageId>, prev_page_id: Option<PageId>) -> PageResult<Option<OverflowPage>> 
    {
        if crate::io::is_empty(data).map_err(PageError::from)?
        {
            if let Some(pg_id) = target_page_id {
                Self::drop_tail(pager, &pg_id)?;
            }
            return Ok(None)
        }

        // Retrieve or create the overflow page
        let mut target_pg = match target_page_id {
            Some(pg_id) => OverflowPage::get(&pg_id, pager),
            None => Self::new_overflow_page(pager)
        }?;

        // Write the data to the page.
        target_pg.write(pager, data)?;

        // We have a pointer to the next page, but there is no more data.
        // We drop the rest of the overflow list.
        if target_pg.get_next().is_some() && crate::io::is_empty(data).map_err(PageError::from)? {
            Self::drop_tail(pager, &target_pg.get_next().unwrap())?;
            target_pg.set_next(None);
        }

        // Get the previous overflow page, if any, and point it to the current page.
        if let Some(prev_page_id) = prev_page_id {
            let mut prev = OverflowPage::get(&prev_page_id, pager)?;
            prev.set_next(Some(target_pg.page_id));
            prev.flush(pager)?;
        }
        
        target_pg.flush(pager)?;

        Ok(Some(target_pg))

    }

    /// Drop all the rest of the pages from the overflow chain starting from from_page_id.
    /// This method opens the page, and drops it.
    fn drop_tail<P: Pager>(pager: &mut P, from_page_id: &PageId) -> PageResult<()> {
        let mut pg = OverflowPage::get(from_page_id, pager)?;
        pager.drop_page(&pg.page_id)?;

        if let Some(next_pg) = pg.get_next() {
            return Self::drop_tail(pager, &next_pg)
        }

        pg.flush(pager)?;
        
        Ok(())
    }

    pub fn read<P: Pager, W: Write>(pager: &mut P, head: &PageId, to: &mut W) -> PageResult<()> {
        let mut pg_cursor: Option<PageId> = Some(*head);

        while let Some(next_pg) = pg_cursor {
            let pg = OverflowPage::get(&next_pg, pager)?;
            to.write_all(&pg.read(pager)?)?;
            pg_cursor = pg.get_next();
        }

        Ok(())
    }

    /// Create a new overflow page.
    fn new_overflow_page<P: Pager>(pager: &mut P) -> PageResult<OverflowPage> 
    {
        OverflowPage::new(pager)
    }
}

#[cfg(test)]
mod tests 
{
    use crate::fixtures::pager_fixture;
    use crate::io::Data;
    use crate::pager::overflow::{Overflow, BlockSize, OverflowHeader};
    use crate::pager::page::result::PageResult;

    use super::OverflowPage;
    
    #[test]
    pub fn test_overflow_page() -> PageResult<()> {
        let data_size = 1_000_000usize;
        let data = crate::fixtures::random_data(data_size);
        let mut pager = pager_fixture(4000usize);
        
        let mut page = OverflowPage::new(&mut pager)?;
        let written = page.write(&mut pager, &mut data.get_cursor_read())?;

        println!("Written: {}", written);
        assert_eq!(BlockSize::from(written), page.header.in_page_size);
        
        let ptr: u64 = page.header.in_page_ptr.into();
        assert_eq!(ptr as usize, OverflowHeader::size_of());

        let data_chunk = page.read(&mut pager)?;
        assert_eq!((&data_chunk)[0..written], (&data)[0..written]);

        Ok(())
    }

    #[test]
    /// Test the data overflow mechanism
    pub fn test_pager_overflow() -> PageResult<()> {
        // Try with 1 MB of overflow data, into 4 kB pages.
        let data_size = 1_000_000usize;
        let mut pager = pager_fixture(4000usize);
        let data = crate::fixtures::random_data(data_size);

        // Schedule an overflow writing
        let pg_id = Overflow::write(&mut pager, &mut data.get_cursor_read(), None)?.unwrap();

        // Retrieve the whole stored data.
        // In this example, the data must have been splitted into several overflow pages. 
        let mut stored_data = Data::new();
        Overflow::read(&mut pager, &pg_id, &mut stored_data)?;
        
        assert_eq!(stored_data, data);

        Ok(())
    }
}