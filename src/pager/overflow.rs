use std::ops::Deref;

use crate::io::{DataBuffer, DataStream};
use crate::io::traits::{OutStream, InStream};

use super::page::{PageSize};
use super::offset::PAGE_BODY_OFFSET;
use super::{id::PageId, offset::PageOffset, page_type::PageType, PagerResult};
use super::traits::{Pager as TraitPager};

/// Header of an overflow page
#[derive(Default)]
pub struct OverflowHeader
{
    pub next: Option<PageId>,
    pub in_page_size: u64
}

impl OutStream for OverflowHeader {
    fn write_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        Ok(self.next.write_to_stream(writer)? + 
        DataStream::<u64>::write(writer, self.in_page_size)?)
    }

    fn write_all_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.next.write_all_to_stream(writer)?;
        DataStream::<u64>::write_all(writer, self.in_page_size)
    }
}

impl InStream for OverflowHeader 
{
    fn read_from_stream<R: std::io::BufRead>(&mut self, reader: &mut R) -> std::io::Result<()> {
        self.next.read_from_stream(reader)?;
        self.in_page_size = DataStream::<u64>::read(reader)?;
        Ok(())
    }
}

impl OverflowHeader 
{
    pub const fn size_of() -> u64 { PageId::size_of() + 8 }
}

pub const OVERFLOW_HEADER_SIZE: u64 = OverflowHeader::size_of();
pub const OVERFLOW_OFFSET: PageOffset = PAGE_BODY_OFFSET;
pub const OVERFLOW_HEADER_OFFSET: PageOffset = OVERFLOW_OFFSET;
pub const OVERFLOW_NEXT_FIELD_OFFSET: PageOffset = OVERFLOW_HEADER_OFFSET;
pub const OVERFLOW_BODY_OFFSET: PageOffset = OVERFLOW_HEADER_OFFSET + OVERFLOW_HEADER_SIZE;

pub fn max_body_size(page_size: PageSize) -> u64 {
    if OVERFLOW_BODY_OFFSET > page_size 
    {
        0
    } else {
        page_size - OVERFLOW_BODY_OFFSET
    }
}

pub struct OverflowCommand 
{
    /// Which page is the source of the overflow.
    pub src_page: PageId,
    /// The offset to write back the overflow page id.
    pub src_offset: PageOffset,
    /// The targetted overflow page if it does already exist.
    pub overflow_page_id: Option<PageId>,
    /// The raw data to write into multiple overflow pages
    pub data: DataBuffer
}

impl OverflowCommand {
    pub fn new(src_page: PageId, src_offset: PageOffset, data: impl Into<DataBuffer>, target_page_id: Option<PageId>) -> Self {
        Self { src_page: src_page, src_offset: src_offset, overflow_page_id: target_page_id, data: data.into() }
    }
}

/// A tool to transfer overflowed data into dedicated pages.
pub struct Overflow {
    /// Page size
    pub page_size: u64,
    /// Commands to execute
    pub commands: Vec<OverflowCommand>
}

/// A tool to retrive the data stored in an overflow page.
struct OverflowData {
    data: DataBuffer
}

impl Deref for OverflowData {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl InStream for OverflowData 
{
    fn read_from_stream<R: std::io::BufRead>(&mut self, read: &mut R) -> std::io::Result<()> 
    {
        read.read_exact(&mut self.data)
    }
}

impl<'a> From<&'a OverflowHeader> for OverflowData {
    fn from(value: &'a OverflowHeader) -> Self {
        Self { data: DataBuffer::with_size(value.in_page_size as usize) }
    }
}

impl<P> From<&P> for Overflow 
where P: TraitPager 
{
    fn from(value: &P) -> Self {
        Self {
            page_size: value.get_page_size(),
            commands: vec![]
       }
    }
}

impl Overflow
{
    /// Write data into overflow pages.
    pub fn write<P: TraitPager>(pager: &mut P, data: &mut DataBuffer, base: Option<PageId>) -> PagerResult<Option<PageId>>
    {
        let mut prev: Option<PageId> = None;
        let mut head: Option<PageId> = None;
        let mut target: Option<PageId> = base;

        while let Some(overflow_page_id) = Self::write_oveflow(pager, data, target, prev)?
        {
            let header = Self::read_header(pager, &overflow_page_id)?;
            target = header.next;

            if head.is_none() {
                head = Some(overflow_page_id);
            }

            prev = Some(overflow_page_id);
        }

        Ok(head)
    }
    
    fn write_oveflow<P: TraitPager>(pager: &mut P, data: &mut DataBuffer, target_page_id: Option<PageId>, prev_page_id: Option<PageId>) -> PagerResult<Option<PageId>> 
    {
        if data.is_empty() 
        {
            return Ok(None);
        }

        // Retrieve the overflow page's id, or create a new one if not set.
        let target_page_id = match target_page_id 
        {
            Some(page_id) => {
                let page_id = pager.open_page(&page_id)?;
                // TODO: Assert page type for safety.
                page_id
            },
            None => Self::new_overflow_page(pager)?
        };

        // Assert this is indeed an overflow page.
        pager.assert_page_type(&target_page_id, &PageType::Overflow)?;

        if let Some(prev_page_id) = prev_page_id {
            let mut prev_header = Self::read_header(pager, &prev_page_id)?;
            prev_header.next = Some(target_page_id);
            
            unsafe {
                Self::write_header_unchecked(pager, &prev_header, &prev_page_id)?;
            }
        }

        unsafe 
        {
            // Get the overflow header.
            let mut header = pager.read_and_instantiate_from_page::<OverflowHeader, _>(&target_page_id, OVERFLOW_HEADER_OFFSET)?;
            let max_overflow_size = max_body_size(pager.get_page_size());
            
            // Store the data chunk in the page up to its maximum available capacity.
            let chunk = data.pop_front(max_overflow_size);
            pager.write_all_to_page(&target_page_id, &chunk, OVERFLOW_BODY_OFFSET)?;
            
            // Write the quantity of bytes stored in the page.
            header.in_page_size = chunk.len() as u64;
            pager.write_all_to_page(&target_page_id, &header, OVERFLOW_HEADER_OFFSET)?;

            // Remove the next pointer reference, if any, and drop the tail of the linked list.
            if header.next.is_some() && data.is_empty()
            {    
                Self::drop_tail(pager, &header.next.unwrap())?;
                header.next = None;
                pager.write_all_to_page(&target_page_id, &header, OVERFLOW_HEADER_OFFSET)?;
            }
        }

        Ok(Some(target_page_id))

    }

    /// Read the overflow header, does not check for page type.
    /// This method does not check the page type.
    unsafe fn write_header_unchecked<P: TraitPager>(pager: &mut P, header: &OverflowHeader, page_id: &PageId) -> PagerResult<()> 
    {
        pager.write_all_to_page(page_id, header, OVERFLOW_HEADER_OFFSET)
    }


    /// Drop all the rest of the pages from the overflow chain starting from from_page_id.
    /// This method opens the page, and drops it.
    fn drop_tail<P: TraitPager>(pager: &mut P, from_page_id: &PageId) -> PagerResult<()> 
    {
        pager.open_page(from_page_id)?;
        
        let header = Self::read_header(pager, from_page_id)?;
        pager.drop_page(from_page_id)?;

        if let Some(next_page_id) = header.next {
            Self::drop_tail(pager, &next_page_id)
        } else {
            Ok(())
        }
    }

    /// Read data from overflow page
    /// This method does not check the page type.
    unsafe fn read_data_unchecked<P: TraitPager>(pager: &mut P, page_id: &PageId, data: &mut OverflowData) -> PagerResult<()> 
    {
        pager.read_from_page(data, page_id, OVERFLOW_BODY_OFFSET)
    }

    /// Read the overflow header, does not check for page type.
    /// This method does not check the page type.
    unsafe fn read_header_unchecked<P: TraitPager>(pager: &mut P, page_id: &PageId) -> PagerResult<OverflowHeader> 
    {
        pager.read_and_instantiate_from_page::<OverflowHeader, _>(&page_id, OVERFLOW_HEADER_OFFSET)
    }

    /// Read the overflow header
    pub fn read_header<P: TraitPager>(pager: &mut P, page_id: &PageId) -> PagerResult<OverflowHeader> {
        pager.assert_page_type(page_id, &PageType::Overflow)?;
        unsafe {
            Self::read_header_unchecked(pager, page_id)
        }
    }

    /// Read all data, does not check for page type.
    /// This method does not check the page type.
    pub unsafe fn read_unchecked<P: TraitPager>(pager: &mut P, page_id: &PageId, acc: &mut DataBuffer) -> PagerResult<()>
    {
        let mut cursor_page_id = *page_id;

        while let Some(next_page_id) = Self::read_overflow(pager, &cursor_page_id, acc)? 
        {
            cursor_page_id = next_page_id;
        };

        Ok(())
    }

    unsafe fn read_overflow<P: TraitPager>(pager: &mut P, page_id: &PageId, acc: &mut DataBuffer) -> PagerResult<Option<PageId>> {
        pager.open_page(page_id)?;

        let header = Self::read_header_unchecked(pager, page_id)?;
        let mut data: OverflowData = OverflowData::from(&header);
        Self::read_data_unchecked(pager, page_id, &mut data)?;
        acc.extend_from_slice(&data);

        Ok(header.next)
    }

    /// Read raw data from the overflow page chain, and store the result in the accumulator.
    pub fn read_raw<P: TraitPager>(pager: &mut P, page_id: &PageId, acc: &mut DataBuffer) -> PagerResult<()> 
    {
        pager.assert_page_type(page_id, &PageType::Overflow)?;
        
        unsafe {
            Self::read_unchecked(pager, page_id, acc)
        }
    }

    /// Read the data and send it to the element.
    pub fn read<P: TraitPager, E: InStream>(pager: &mut P, to: &mut E, page_id: &PageId, base: Option<&mut DataBuffer>) -> PagerResult<()> {
        let mut rd = DataBuffer::new();
        let acc = base.unwrap_or(&mut rd);
        Self::read_raw(pager, page_id, acc)?;
        to.read_from_stream(acc)?;
        Ok(())
    }

    /// Read the data, and create the element.
    pub fn read_and_instantiate<P: TraitPager, E: InStream + Default>(pager: &mut P, page_id: &PageId, base: Option<&mut DataBuffer>) -> PagerResult<E> {
        let mut data = E::default();
        Self::read(pager, &mut data, page_id, base)?;
        Ok(data)
    }

    /// Create a new overflow page.
    fn new_overflow_page<P: TraitPager>(pager: &mut P) -> PagerResult<PageId> 
    {
        let header = OverflowHeader::default();
        let page_id = pager.new_page(PageType::Overflow)?;
        unsafe {
            pager.write_all_to_page(&page_id, &header, OVERFLOW_HEADER_OFFSET)?;
        }
        Ok(page_id)
    }
}

#[cfg(test)]
mod tests 
{
    use crate::io::{DataBuffer};
    use crate::pager::overflow::{Overflow};
    use crate::pager::{Pager, PagerResult};
    
    #[test]
    /// Test the data overflow mechanism
    pub fn test_pager_overflow() -> PagerResult<()> 
    {
        // Try with 1 MB of overflow data, into 4 kB pages.
        let data_size = 1_000_000usize;
        let mut pager = Pager::new(DataBuffer::new(), 4000);
        let data = crate::fixtures::random_raw_data(data_size);

        // Schedule an overflow writing
        let page_id = Overflow::write(&mut pager, &mut data.clone(), None)?.unwrap();

        // Retrieve the whole stored data.
        // In this example, the data must have been splitted into two overflow pages. 
        let mut stored_data = DataBuffer::with_size(data_size);
        Overflow::read(&mut pager, &mut stored_data, &page_id, None)?;
        assert_eq!(stored_data, data);

        Ok(())
    }
}