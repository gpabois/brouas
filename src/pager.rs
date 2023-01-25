use std::{io::{Read, Write, Seek, SeekFrom}, ops::DerefMut};

use crate::{buffer::{Buffer, BufferCellIterator}, utils::Counter};

use self::page::{BufPage, Page, traits::{ReadPage, WritePage}};

pub mod page;
// pub mod overflow;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    BufferError(crate::buffer::Error),
    IoError(std::io::Error)
}

impl Into<std::io::Error> for Error {
    fn into(self) -> std::io::Error {
        match self {
            Error::BufferError(err) => std::io::Error::new(std::io::ErrorKind::OutOfMemory, format!("memory buffer error: {:?}", err)),
            Error::IoError(err) => err,
        }
    }
}

impl From<crate::buffer::Error> for Error {
    fn from(err: crate::buffer::Error) -> Self {
        Self::BufferError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

pub type PageId = u64;

pub mod traits {
    use super::{Result, PageId};

    pub trait Pager<'a> {
        type Page;
        
        fn new_page(&'a self, ptype: u8) -> Result<Self::Page>;
        fn get_page(&'a self, pid: PageId)  -> Result<Self::Page>;
        fn drop_page(&self, pid: PageId) -> Result<()>;
        fn flush(&self) -> Result<()>;
    }
}

pub const PAGE_SIZE: usize = 16_000;

pub const RESERVED: usize = 10;
pub const FREE_PAGE: u8 = 0x00;
pub const OVERFLOW_PAGE: u8 = 0xFF;

pub struct BufPageIterator<'buffer> {
    cells: BufferCellIterator<'buffer>
}

impl<'buffer> BufPageIterator<'buffer> {
    pub fn new(cells: BufferCellIterator<'buffer>) -> Self {
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
            Some(cell) => cell.try_into_array::<u8>().map(Page::from)
        }
    }
}

pub struct Pager<'buffer, Stream>
{
    pool: Buffer,
    io: std::cell::RefCell<Stream>,
    counter: Counter,
    _pht: std::marker::PhantomData<&'buffer ()>
}

impl<'buffer, Stream> self::traits::Pager<'buffer> for Pager<'buffer, Stream>
where Stream: Read + Write + Seek {

    type Page = BufPage<'buffer>;
    
    /// Create a new page
    fn new_page(&'buffer self, ptype: u8) -> Result<Self::Page> 
    {
        let pid = self.counter.inc();
        let area = self.pool.alloc_array_uninit::<u8>(PAGE_SIZE)?;
        let page = Page::new(pid, ptype, area);
        Ok(page)
    }

    /// Get a page by its index
    fn get_page(&'buffer self, index: PageId) -> Result<Self::Page> {
        // We check if the page is already stored in memory
        if let Some(page) = self.iter().find(|page| page.get_id() == index)
        {
            Ok(page)
        } 
        // We need to load it from the stream
        else {
            self.io.borrow_mut().seek(SeekFrom::Start(self.reserved() as u64))?;
            let mut page = Page::from(self.pool.alloc_array_uninit::<u8>(PAGE_SIZE)?);
            self.io.borrow_mut().deref_mut().read_exact(page.deref_mut())?;
            Ok(page)
        }
    }

    /// Drop the page
    fn drop_page(&self, pid: PageId) -> Result<()> {
        self.get_page(pid)?.set_type(FREE_PAGE);
        Ok(())
    }

    fn flush(&self) -> Result<()> {
        for mut page in self.iter_upserted_pages() {
            let offset = (RESERVED + page.get_size() * (page.get_id() as usize)) as u64;
            self.io.borrow_mut().seek(SeekFrom::Start(offset))?;
            self.io.borrow_mut().write_all(&page)?;
            page.borrow_mut_data().drop_modification_flag();
        }

        Ok(())
    }
}

impl<'buffer, Stream> Pager<'buffer, Stream>
where Stream: Read + Write + Seek
{
    /// Reserved size of the pager header
    fn reserved(&self) -> usize {
        0
    }

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
        self.iter().filter(|page| page.borrow_data().is_modified())
    }

    /// Iterate over in memory pages
    pub fn iter(&self) -> impl Iterator<Item=BufPage> {
        BufPageIterator::new(self.pool.iter())
    }
}

#[cfg(test)]
mod tests {
    use std::{io::{Write, Read}};
    use crate::{io::{InMemory, Data}, fixtures, pager::page::traits::{WritePage, ReadPage}};
    use super::{traits::Pager, PageId};


    #[test]
    fn test_pager() -> super::Result<()> {
        let pager = super::Pager::new(InMemory::new(), 10);
        
        let data_size: usize = 1000;
        let random = fixtures::random_data(data_size);
        
        let pid: PageId;
        {
            let mut page = pager.new_page(0x10)?;
            page.deref_mut_body().write_all(&random)?;
            pid = page.get_id();
        }

        let mut stored = Data::with_size(data_size);
        let page = pager.get_page(pid)?;
        page.deref_body().read(&mut stored)?;
        
        assert_eq!(random, stored);

        Ok(())
    }
}