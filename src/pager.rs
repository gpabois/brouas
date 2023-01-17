use std::{io::{Read, Write, Seek}, ops::{Index, Deref}};

use crate::buffer::{BufferPool, BufferCell};

use self::page::{BrouasPage};

pub mod page;
// pub mod bptree;

pub type PageId = u64;
pub struct Pager<Stream>
{
    pool: BufferPool<BrouasPage>,
    io: Stream,

}

impl<Stream> Pager<Stream>
where Stream: Read + Write + Seek
{
    /// Create a pager
    /// io: The stream to read and write into
    /// buffer_size: number of pages that can be stored in memory
    pub fn new(io: Stream, buffer_size: usize) -> Self {
        Self {
            io,
            pool: BufferPool::new(buffer_size)
        }
    }

    /// Return an iterator over upserted pages.
    pub fn iter_upserted_pages(&self) -> impl Iterator<Item=BufferCell<BrouasPage>> {
        self.pool.iter().filter(|cell| cell.is_modified())
    }

    pub fn get_page(&self, index: PageId) -> BufferCell<BrouasPage> {
        // We check if the page is already stored in memory
        if let Some(page) = self.pool.iter().find(|cell| cell.get_id() == index)
        {
            page
        } 
        // We need to load it from the stream
        else {
            
        }
    }
}