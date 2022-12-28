use crate::{pager::traits::Pager, io::traits::{InStream, OutStream}};

use super::{page::{id::PageId, offset::PageOffset, result::PageResult, page_type::PageType}};

#[derive(Default)]
pub struct FreeHeader 
{
    next: Option<PageId>
}

impl InStream for FreeHeader {
    fn read_from_stream<R: std::io::BufRead>(&mut self, read: &mut R) -> std::io::Result<()> {
        self.next.read_from_stream(read)
    }
}

impl OutStream for FreeHeader {
    fn write_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        self.next.write_to_stream(writer)
    }

    fn write_all_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.next.write_all_to_stream(writer)
    }
}

pub struct FreePage {
    page_id: PageId,
    base: PageOffset,
    header: FreeHeader
}

impl FreePage 
{    
    pub fn new<P: Pager>(pager: &mut P, page_id: PageId) -> PageResult<Self> 
    {
        unsafe {
            pager.change_page_type(&page_id, PageType::Free)?;
        }

        let base = pager.get_body_ptr(&page_id)?;
        let header = FreeHeader::default();

        Ok(Self {
            page_id: page_id,
            base:    base,
            header:  header
        })
    }

    pub fn get<P: Pager>(page_id: &PageId, pager: &mut P) -> PageResult<Self> {
        unsafe {
            pager.open_page(page_id)?;
            pager.assert_page_type(page_id, &PageType::Free)?;
            
            let base = 0u64.into();
            let header = pager.read_and_instantiate_from_page::<FreeHeader, _>(page_id, base)?;
            
            Ok(Self {
                page_id: *page_id,
                base:     base,
                header:   header
            })
        }
    }

    pub fn flush<P: Pager>(&mut self, pager: &mut P) -> PageResult<()> {
        unsafe {
            pager.write_all_to_page(&self.page_id, &self.header, self.base)
        }
    }
 
    pub fn set_next(&mut self, next: Option<PageId>) {
        self.header.next = next;
    }

    pub fn get_next(&self) -> Option<PageId> {
        self.header.next
    }
}

pub struct Allocator;

impl Allocator
{   
    /// Allocate a new page, if there are any free pages available.
    /// This methods opens the page, if any.
    pub fn alloc<P: Pager>(pager: &mut P) -> PageResult<Option<PageId>> 
    {
        Self::pop(pager)
    }

    /// Free the page, and returns the new head of the free stack list.
    /// This methods does not open the page, nor does it close it or flush it.
    pub fn free<P: Pager>(pager: &mut P, page_id: &PageId) -> PageResult<()> 
    {
        // Create a free page from an existing one.
        let mut pg = FreePage::new(pager, *page_id)?;
        
        // Save the new state
        pg.flush(pager)?;

        // Push it on top of the stack
        Self::push(pager, page_id)?;

        Ok(())
    }
    
    /// Pop a free page, and opens it.
    fn pop<P: Pager>(pager: &mut P) -> PageResult<Option<PageId>> 
    {
        if let Some(page_id) = pager.get_freelist_head() {
            pager.open_page(&page_id)?;
            let head = FreePage::get(&page_id, pager)?;
            pager.set_freelist_head(head.get_next());
            Ok(Some(page_id))
        } else {
            Ok(None)
        }
    }

    fn push<P: Pager>(pager: &mut P, page_id: &PageId) -> PageResult<()> 
    {
        match Self::get_free_head(pager)? {
            Some(mut pg) => {
                pg.set_next(Some(*page_id));
                pg.flush(pager)?;
            }
            None => {}
        }
        
        pager.set_freelist_head(Some(*page_id));
        
        Ok(())
    }

    fn get_free_head<P: Pager>(pager: &mut P) -> PageResult<Option<FreePage>> 
    {
        pager
        .get_freelist_head()
        .map(|head| FreePage::get(&head, pager))
        .transpose()
    }
}


