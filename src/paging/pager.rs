use std::{io::{Read, Write, Seek, SeekFrom}, ops::DerefMut};

use crate::{buffer::{Buffer, BufCellIterator}, utils::{Counter, cell::TryCell, slice::{IntoSection, CloneSection}, borrow::TryBorrowMut}};

use super::{page::{BufPage, PageSectionType, Page}, error::Error, result::Result};


pub type PageId = u64;

pub mod traits {
    use super::{PageId};
    use std::result::Result;

    pub trait Pager<'a> {
        type Error;
        type Page;
        
        /// Create a new page
        fn new_page(&'a self, ptype: u8) -> Result<Self::Page, Self::Error>;

        /// Returns a cell to a page that can be upgraded to a mutable/immutable reference.
        fn get_page(&'a self, pid: PageId) -> Result<Self::Page, Self::Error>;

        /// Drop the page
        fn drop_page(&self, pid: PageId) -> Result<(), Self::Error>;

        /// Flush upserted pages into the stream
        fn flush(&self) -> Result<(), Self::Error>;
    }
}

pub const PAGE_SIZE: usize = 16_000;
pub const RESERVED: usize = 10;
pub const FREE_PAGE: u8 = 0x00;
pub const OVERFLOW_PAGE: u8 = 0xFF;

pub struct BufPageIterator<'buffer> {
    cells: BufCellIterator<'buffer>
}

impl<'buffer> BufPageIterator<'buffer> {
    pub fn new(cells: BufCellIterator<'buffer>) -> Self {
        Self {
            cells
        }
    }
}

impl<'buffer> Iterator for BufPageIterator<'buffer> {
    type Item = BufPage<'buffer>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.cells.next() {
            None => None,
            Some(cell) => cell.try_into_array::<u8>().map(Self::Item::from)
        }
    }
}

pub struct BufPager<'buffer, Stream>
{
    pool: Buffer,
    io: std::cell::RefCell<Stream>,
    counter: Counter,
    _pht: std::marker::PhantomData<&'buffer ()>
}

impl<'buffer, Stream> self::traits::Pager<'buffer> for BufPager<'buffer, Stream>
where Stream: Read + Write + Seek {
    type Error = Error;
    type Page = BufPage<'buffer>;

    /// Create a new page
    fn new_page(&'buffer self, ptype: u8) -> Result<Self::Page> 
    {
        let pid = self.counter.inc();
        let area = self.pool.alloc_array_uninit::<u8>(PAGE_SIZE)?;
        let page = BufPage::try_new(pid, ptype, area)?;
        Ok(page)
    }

    /// Get a page by its index
    fn get_page(&'buffer self, index: PageId) -> Result<Self::Page> {
        // We check if the page is already stored in memory
        if let Some(page) = self.iter().find(|page| page.try_borrow().unwrap().get_id() == index)
        {
            Ok(page)
        } 
        // We need to load it from the stream
        else {
            self.io.borrow_mut().seek(SeekFrom::Start(RESERVED as u64))?;
            let page = self.pool.alloc_array_uninit::<u8>(PAGE_SIZE).map(Page::from)?;

            self.io.borrow_mut().deref_mut().read_exact(
                page
                .clone_section(PageSectionType::All)
                .try_borrow_mut()?
                .as_mut()
            )?;

            Ok(page)
        }
    }

    /// Drop the page
    fn drop_page(&self, pid: PageId) -> Result<()> {
        self.get_page(pid)?
        .try_borrow_mut()?
        .set_type(FREE_PAGE);
        Ok(())
    }

    fn flush(&self) -> Result<()> {
        for mut page in self.iter_upserted_pages() {
            let offset = (RESERVED + page.try_borrow()?.get_size() * (page.try_borrow()?.get_id() as usize)) as u64;
            self.io.borrow_mut().seek(SeekFrom::Start(offset))?;
            self.io.borrow_mut().write_all(
                page.try_borrow_mut()?.into_section(PageSectionType::All).as_mut()
            )?;
            page.ack_upsertion();
        }

        Ok(())
    }

}

impl<'buffer, Stream> BufPager<'buffer, Stream>
where Stream: Read + Write + Seek
{
    /// Create a pager
    /// io: The stream to read and write into
    /// buffer_size: number of pages that can be stored in memory
    pub fn new(io: Stream, buffer_size: usize) -> Self {
        Self {
            io: std::cell::RefCell::new(io),
            pool: Buffer::new_by_array::<u8>(PAGE_SIZE, buffer_size),
            counter: Default::default(),
            _pht: Default::default()
        }
    }

    /// Return an iterator over upserted pages.
    pub fn iter_upserted_pages(&self) -> impl Iterator<Item=BufPage> {
        self.iter().filter(|page| page.is_upserted())
    }

    /// Iterate over in memory pages
    pub fn iter(&self) -> impl Iterator<Item=BufPage> {
        BufPageIterator::new(self.pool.iter())
    }

}

#[cfg(test)]
mod tests {
    use std::{io::{Write, Read}, ops::DerefMut};
    use crate::{io::{InMemory, Data}, fixtures, paging::page::{PageSectionType}, utils::{cell::TryCell, slice::{BorrowSection, IntoSection, CloneSection}, borrow::TryBorrowMut}};
    use super::{traits::Pager, PageId};


    #[test]
    fn test_pager() -> super::Result<()> {
        let pager = super::BufPager::new(InMemory::new(), 10);
        
        let data_size: usize = 1000;
        let random = fixtures::random_data(data_size);
        
        let pid: PageId;
        {
            let page = pager.new_page(0x10)?;
            pid = page.try_borrow()?.get_id();
            
            page
            .clone_section(PageSectionType::Body)
            .try_borrow_mut()?
            .deref_mut()
            .write_all(&random)?;
        }

        let mut stored = Data::with_size(data_size);
        let page = pager.get_page(pid)?.try_borrow()?;
        page.clone_section(PageSectionType::Body).as_ref().read(&mut stored)?;
        
        assert_eq!(random, stored);

        Ok(())
    }
}