use std::io::{Seek, SeekFrom, Write, Read};

use crate::io::DataBuffer;
use crate::io::traits::InStream;

use super::PagerHeader;
use super::id::PageId;
use super::traits::PagerStream;
use super::page::Page;

/// Allows the data buffer to act as an input stream for the pager.
impl PagerStream for DataBuffer 
{
    fn write_page(&mut self, page_id: &PageId, page: &Page) -> std::io::Result<()> 
    {
        let offset = (*page_id) * page.get_page_size();    
        let size = offset + page.get_page_size();
        self.increase_size_if_necessary(size as usize);
        
        let mut stream = self.get_buf_write();
        stream.seek(SeekFrom::Start(offset))?;
        stream.write_all(page)?;
        
        Ok(())
    }

    fn read_page(&mut self, page_id: &super::id::PageId, page: &mut Page) -> std::io::Result<()> 
    {
        let offset = (*page_id) * page.get_page_size();
        let mut stream = self.get_buf_read();
        stream.seek(SeekFrom::Start(offset))?;
        stream.read_exact(page)?;
        Ok(())
    }

    fn read_header(&mut self) -> std::io::Result<super::PagerHeader> 
    {
        let mut stream = self.get_buf_read();
        stream.seek(SeekFrom::Start(0))?;
        let mut header = PagerHeader::default();
        header.read_from_stream(&mut stream)?;
        Ok(header)
    }
}