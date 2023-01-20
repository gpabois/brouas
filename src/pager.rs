use std::{io::{Read, Write, Seek, SeekFrom}, ops::DerefMut, borrow::BorrowMut};

use crate::{buffer::{BufferPool, BufferCell}, utils::Counter};

use self::page::{BrouasPage};

pub mod page;
pub mod overflow;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    BufferError(crate::buffer::Error),
    IoError(std::io::Error)
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
    use super::page::BrouasPageCell;
    use super::{Result, PageId};

    pub trait Pager {
        fn new_page(&self, ptype: u8)    -> Result<BrouasPageCell>;
        fn get_page(&self, pid: PageId)  -> Result<BrouasPageCell>;
        fn drop_page(&self, pid: PageId) -> Result<()>;
        fn flush(&self) -> Result<()>;
    }
}
pub const RESERVED: usize = 10;

pub const FREE_PAGE: u8 = 0x00;
pub const OVERFLOW_PAGE: u8 = 0xFF;

pub struct Pager<Stream>
{
    pool: BufferPool<BrouasPage>,
    io: std::cell::RefCell<Stream>,
    counter: Counter
}

impl<Stream> self::traits::Pager for Pager<Stream>
where Stream: Read + Write + Seek {

    /// Create a new page
    fn new_page(&self, ptype: u8) -> Result<BufferCell<BrouasPage>> 
    {
        let pid = self.counter.inc();
        let mut page = self.pool.alloc_uninit()?;
        page.init(pid, ptype);
        Ok(page)
    }

    /// Get a page by its index
    fn get_page(&self, index: PageId) -> Result<BufferCell<BrouasPage>> {
        // We check if the page is already stored in memory
        if let Some(page) = self.pool.iter().find(|cell| cell.get_id() == index)
        {
            Ok(page)
        } 
        // We need to load it from the stream
        else {
            self.io.borrow_mut().seek(SeekFrom::Start(self.reserved() as u64))?;
            let mut page = self.pool.alloc_uninit()?;
            page.deref_mut().read(self.io.borrow_mut().deref_mut())?;
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
            let offset = (RESERVED + BrouasPage::get_size() * (page.get_id() as usize)) as u64;
            self.io.borrow_mut().seek(SeekFrom::Start(offset))?;
            std::io::copy(
                &mut page.get_reader(),
                self.io.borrow_mut().deref_mut()
            )?;
            page.drop_modification_flag();
        }

        Ok(())
    }
}

impl<Stream> Pager<Stream>
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
            pool: BufferPool::new(buffer_size),
            counter: Default::default()
        }
    }

    /// Return an iterator over upserted pages.
    pub fn iter_upserted_pages(&self) -> impl Iterator<Item=BufferCell<BrouasPage>> {
        self.pool.iter().filter(|cell| cell.is_modified())
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Write, Read};

    use crate::{io::{InMemory, Data}, fixtures};

    use super::{traits::Pager, PageId};


    #[test]
    fn test_pager() -> super::Result<()> {
        let pager = super::Pager::new(InMemory::new(), 10);
        
        let data_size: usize = 1000;
        let random = fixtures::random_data(data_size);
        
        let pid: PageId;
        {
            let mut page = pager.new_page(0x10)?;
            pid = page.get_id();
            page.get_writer().write(&random)?;
        }

        let mut stored = Data::with_size(data_size);
        let page = pager.get_page(pid)?;
        page.get_reader().read(&mut stored)?;
        
        assert_eq!(random, stored);


        Ok(())
    }
}