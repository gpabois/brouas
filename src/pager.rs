use std::{io::{Read, BufReader, Write, BufWriter, Cursor}, alloc::{Layout, alloc_zeroed, dealloc}, mem::size_of, collections::HashMap, ops::Add};

use self::{page_header::PageHeader, page_id::PageId, btree_page::page_header::BPTreeHeader, page_type::PageType};

pub mod page_id;
pub mod page_type;
pub mod page_header;
pub mod page_nonce;
pub mod btree_page;

pub enum PagerError
{
    IOError(std::io::Error),
    WrongPageType{expecting: PageType, got: PageType}
}

impl From<std::io::Error> for PagerError {
    fn from(err: std::io::Error) -> Self {
        Self::IOError(err)
    }
}

pub type PagerResult<T> = std::result::Result<T, PagerError>;

pub struct Page
{
    /// Deserialized page header
    header: PageHeader,
    /// Base offset of the page (including raw header data)
    base: u64,
}

pub struct PagerHeader
{
    version:    u64,
    page_size:  u64,
    page_count: u64
}


pub struct UnsafePage(Layout, *mut u8);

impl UnsafePage
{
    pub fn alloc(page_size: u64) -> Self {
        let layout = std::alloc::Layout::from_size_align(page_size as usize, size_of::<u8>()).unwrap();
        unsafe {
            UnsafePage(layout, std::alloc::alloc_zeroed(layout))
        }
    }

    unsafe fn get_mut_raw(&self) -> &mut [u8]
    {
        std::slice::from_raw_parts_mut(self.1, self.0.size())
    }

    fn get_buf_read(&self) -> BufReader<Cursor<&mut [u8]>>
    {
        unsafe {
            BufReader::new(Cursor::new(self.get_mut_raw()))
        }
    }

    pub fn assert_type(&self, expecting_page_type: &PageType) -> PagerResult<()> {
        let got_page_type = PageHeader::seek_page_type(&mut self.get_buf_read()).map_err(PagerError::from)?;
        if *expecting_page_type != got_page_type {
            return Err(PagerError::WrongPageType { expecting: *expecting_page_type, got: got_page_type })
        }

        Ok(())
    }
    
    /// Write the btree node header, if the page is a BTree Node
    pub fn write_btree_node_header(&self, header: BPTreeHeader) -> std::result::Result<(), PagerError>
    {
        unsafe {
            let buffer = std::slice::from_raw_parts_mut(self.1, self.0.size());
            let mut buffer = BufWriter::new(Cursor::new(buffer));
            self.assert_type(&PageType::BTree)?;
            
            BPTreeHeader::seek(&mut buffer).map_err(PagerError::from)?;
            //header.write_to_buffer(&mut buffer).map_err(PagerError::from)?;
            Ok(())
        }
    }
    /// Write the page header
    pub fn write_page_header(&self, header: PageHeader) -> PagerResult<usize>
    {
        unsafe {
            let buffer = std::slice::from_raw_parts_mut(self.1, self.0.size());
            let mut buffer = BufWriter::new(Cursor::new(buffer));
            header.write_to_buffer(&mut buffer).map_err(PagerError::from)
        }
    }
}

impl Drop for UnsafePage {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.1, self.0);
        }
    }
}

pub struct PageBuffer {
    page_size: u64,
    index: HashMap<PageId, UnsafePage>,
}

impl PageBuffer
{
    pub fn new(page_size: u64, capacity: u64) -> Self
    {
        Self {
            page_size: page_size,
            index: Default::default()
        }
    }

    pub unsafe fn alloc_page_space_unchecked(&mut self, page_id: &PageId) -> &UnsafePage
    {
        match self.get_page_unchecked(page_id) {
            None => {
                let page = UnsafePage::alloc(self.page_size);
                self.index.insert(page_id.clone(), page);
                self.index.get(page_id).unwrap()
            },
            Some(page) => page
        }
        
    }

    pub unsafe fn get_page_unchecked(&self, page_id: &PageId) -> Option<&UnsafePage>
    {
        self.index.get(page_id)
    }

}

pub struct Pager
{
    header: PagerHeader
}