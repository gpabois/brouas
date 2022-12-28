use std::{alloc::Layout, mem::size_of, io::{SeekFrom, BufWriter, Cursor, BufReader, Seek, Write}, ops::{Deref, DerefMut}};

use crate::io::traits::{InStream, OutStream};

use self::{header::PageHeader, offset::PageOffset, page_type::PageType, id::PageId, metadata::PageMetadata, size::PageSize, nonce::PageNonce};

mod header;

pub mod id;
pub mod nonce;
pub mod offset;
pub mod page_type;
pub mod result;
pub mod error;
pub mod metadata;
pub mod size;


pub const MIN_PAGE_SIZE: usize = PageHeader::size_of();

pub struct Page 
{
    layout: Layout, 
    ptr: *mut u8, 
    pub modified: bool,
    header: PageHeader
}

impl Deref for Page 
{
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe {
            self.get_mut_raw()
        }
    }
}

impl DerefMut for Page 
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            self.get_mut_raw()
        }
    }
}

impl InStream for Page {
    fn read_from_stream<R: std::io::BufRead>(&mut self, read: &mut R) -> std::io::Result<()> {
        unsafe {
            read.read_exact(self.get_mut_raw())
        }
    }
}

impl Page
{
    pub unsafe fn alloc(page_size: PageSize) -> Self 
    {
        let layout = std::alloc::Layout::from_size_align(page_size.into(), size_of::<u8>()).unwrap();
        
        Page
        {
            layout: layout, 
            ptr: std::alloc::alloc_zeroed(layout), 
            modified: false,
            header: Default::default()
        }
    }

    pub fn new(page_id: PageId, page_size: PageSize, page_type: PageType) -> Self
    {
        unsafe 
        {
            let mut page = Self::alloc(page_size);
            page.header.page_type = page_type;
            page.header.id = page_id;
            page.header.nonce = PageNonce::new();
            page
        }
    }

    pub fn flush<W: Write>(&mut self, writer: &mut W) -> std::io::Result<()> {
        // Write the header into the pagen and write the whole thing into the stream
        unsafe {
            self.write_all(&self.header.clone(), 0u32)?;
            writer.write_all(self.get_mut_raw())?;
            self.modified = false;
            Ok(())
        }
    }

    pub unsafe fn read<D: InStream>(&self, to: &mut D, offset: impl Into<PageOffset>) -> std::io::Result<()> 
    {
        let mut reader = self.get_buf_read();
        reader.seek(SeekFrom::Start(offset.into().into()))?;
        to.read_from_stream(&mut reader)
    }

    pub unsafe fn write<D: OutStream>(&mut self, data: &D, offset: impl Into<PageOffset>) -> std::io::Result<usize> 
    {
        self.modified = true;
        let mut writer = self.get_buf_write();
        writer.seek(SeekFrom::Start(offset.into().into()))?;
        data.write_to_stream(&mut writer)
    }

    pub unsafe fn write_all<D: OutStream>(&mut self, data: &D, offset: impl Into<PageOffset>) -> std::io::Result<()> 
    {
        let mut writer = self.get_buf_write();
        writer.seek(SeekFrom::Start(offset.into().into()))?;
        data.write_all_to_stream(&mut writer)
    }

    unsafe fn get_mut_raw(&self) -> &mut [u8]
    {
        std::slice::from_raw_parts_mut(self.ptr, self.layout.size())
    }

    pub fn get_buf_write(&mut self) -> BufWriter<Cursor<&mut [u8]>> 
    {
        BufWriter::new(Cursor::new(self))       
    }

    pub fn get_buf_read(&self) -> BufReader<Cursor<&[u8]>>
    {
        BufReader::new(Cursor::new(self))
    }
    
    pub fn get_size(&self) -> PageSize 
    {
        (self.layout.size() as u64).into()
    }

    pub fn get_type(&self) -> PageType {
        self.header.page_type
    }

    pub unsafe fn set_type(&mut self, page_type: PageType) {
        self.header.page_type = page_type;
        self.modified = true;
    }

    /// Return the body ptr
    pub fn get_body_ptr(&self) -> PageOffset {
        self.header.body_ptr
    }

    pub fn get_metadata(&self) -> PageMetadata {
        self.header.clone().into()
    }

    pub fn get_id(&self) -> PageId {
        self.header.id
    }
}

impl Drop for Page 
{
    fn drop(&mut self) 
    {
        unsafe {
            std::alloc::dealloc(self.ptr, self.layout);
        }
    }
}
