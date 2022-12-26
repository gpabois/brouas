use std::{alloc::Layout, mem::size_of, io::{SeekFrom, BufWriter, Cursor, BufReader, Seek}, ops::{Deref, DerefMut}};

use crate::io::traits::{InStream, OutStream};

use super::{PagerResult, header::PageHeader, PagerError, offset::PageOffset};

pub type PageSize = u64;

pub struct Page 
{
    layout: Layout, 
    ptr: *mut u8, 
    pub modified: bool 
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

impl Page
{
    pub fn get_page_size(&self) -> PageSize 
    {
        (self.layout.size() as u64).into()
    }

    pub unsafe fn alloc(page_size: u64) -> Self 
    {
        let layout = std::alloc::Layout::from_size_align(page_size as usize, size_of::<u8>()).unwrap();
        
        unsafe 
        {
            Page
            {
                layout: layout, 
                ptr: std::alloc::alloc_zeroed(layout), 
                modified: false 
            }
        }
    }

    pub fn new(page_size: u64, header: PageHeader) -> PagerResult<Self> 
    {
        unsafe 
        {
            let mut page = Self::alloc(page_size);
            page.write(&header, &0u64)?;
            Ok(page)
        }
    }

    pub unsafe fn read<D: InStream>(&self, to: &mut D, raw_offset: &PageOffset) -> PagerResult<()> 
    {
        let mut reader = self.get_buf_read();
        reader.seek(SeekFrom::Start(*raw_offset))?;
        to.read_from_stream(&mut reader).map_err(PagerError::from)
    }

    pub unsafe fn write<D: OutStream>(&mut self, data: &D, raw_offset: &PageOffset) -> PagerResult<usize> 
    {
        self.modified = true;
        let mut writer = self.get_buf_write();
        writer.seek(SeekFrom::Start(*raw_offset))?;
        data.write_to_stream(&mut writer).map_err(PagerError::from)
    }

    pub unsafe fn write_all<D: OutStream>(&mut self, data: &D, raw_offset: &PageOffset) -> PagerResult<()> 
    {
        let mut writer = self.get_buf_write();
        writer.seek(SeekFrom::Start(*raw_offset))?;
        data.write_all_to_stream(&mut writer).map_err(PagerError::from)
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
}

impl Drop for Page {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.ptr, self.layout);
        }
    }
}
