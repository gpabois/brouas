use std::io::{Read, Write, Seek};

use crate::io::traits::{OutStream, InStream};

use super::page::Page;
use super::page::{id::PageId, size::PageSize};

pub trait PagerStorage {
    fn flush_page(
        &mut self, 
        page_id: PageId, 
        page_size: PageSize, 
        page: &mut Page,
        base: u64
    ) -> std::io::Result<()>;
    
    fn fetch_page(
        &mut self, 
        page_id: PageId, 
        page_size: PageSize, 
        base: u64
    ) -> std::io::Result<Page>;
    
    fn flush_header<O: OutStream>(&mut self, header: &O, base: u64) -> std::io::Result<()>;
    fn fetch_header<I: InStream>(&mut self, header: &mut I, base: u64) -> std::io::Result<()>;
}

pub struct PagerStream<S: Read + Write + Seek>(S);

impl<S: Read + Write + Seek> PagerStream<S> {
    pub fn new(stream: S) -> Self {
        Self(stream)
    }

    fn page_offset(page_id: PageId, page_size: PageSize, base: u64) -> u64 {
        let id: u64 = page_id.into();
        let size: u64 = page_size.into();

        let offset = base.wrapping_add(id.wrapping_sub(1).wrapping_mul(size));
        offset        
    }
}

impl<S: Read + Write + Seek> PagerStorage for PagerStream<S> 
{
    fn flush_page(
        &mut self, 
        page_id: PageId, 
        page_size: PageSize, 
        page: &mut Page,
        base: u64
    ) -> std::io::Result<()> {
        let offset = Self::page_offset(page_id, page_size, base);
        self.0.seek(std::io::SeekFrom::Start(offset))?;
        page.flush(&mut self.0)
    }

    fn fetch_page(
        &mut self, 
        page_id: PageId, 
        page_size: PageSize, 
        base: u64
    ) -> std::io::Result<Page> {
        let offset = Self::page_offset(page_id, page_size, base);
        self.0.seek(std::io::SeekFrom::Start(offset))?;
        Page::load(page_size, &mut self.0)
    }

    fn flush_header<O: OutStream>(&mut self, header: &O, base: u64) -> std::io::Result<()> {
        self.0.seek(std::io::SeekFrom::Start(base))?;
        header.write_all_to_stream(&mut self.0)
    }

    fn fetch_header<I: InStream>(&mut self, header: &mut I, base: u64) -> std::io::Result<()> {
        self.0.seek(std::io::SeekFrom::Start(base))?;
        header.read_from_stream(&mut self.0)
    }
}