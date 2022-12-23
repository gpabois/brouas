use crate::{pager::TraitPager, io::traits::{InStream, OutStream}};
use super::{id::PageId, change_page_type, PagerResult, page_type::PageType, offset::PAGE_BODY_OFFSET, PagerHeader};

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

impl FreeHeader 
{
    pub fn get<P: TraitPager>(pager: &P, page_id: &PageId) -> PagerResult<Self> {
        pager.assert_page_type(page_id, &PageType::Free)?;
        unsafe {
            Self::get_unchecked(pager, page_id)
        }
    }

    pub fn set<P: TraitPager>(&self, pager: &mut P, page_id: &PageId) -> PagerResult<()> {
        pager.assert_page_type(page_id, &PageType::Free)?;
        unsafe {
            self.set_unchecked(pager, page_id)
        }       
    }

    pub unsafe fn get_unchecked<P: TraitPager>(pager: &P, page_id: &PageId) -> PagerResult<Self> 
    {
        pager.read_and_instantiate_from_page::<FreeHeader, _>(page_id, PAGE_BODY_OFFSET)
    }

    pub unsafe fn set_unchecked<P: TraitPager>(&self, pager: &mut P, page_id: &PageId) -> PagerResult<()> 
    {
        pager.write_all_to_page(page_id, self, PAGE_BODY_OFFSET)
    }
}

pub struct Allocator;

impl Allocator
{   
    /// Allocate a new page, if there are any free pages available.
    /// This methods opens the page, if any.
    pub fn alloc<P: TraitPager>(pager: &mut P) -> PagerResult<Option<PageId>> 
    {
        Self::pop(pager)
    }

    /// Free the page, and returns the new head of the free stack list.
    /// This methods does not open the page, nor does it close it or flush it.
    pub fn free<P: TraitPager>(pager: &mut P, page_id: &PageId) -> PagerResult<()> 
    {
        unsafe 
        {
            // Change the page type to free
            change_page_type(pager, page_id, PageType::Free)?;
        }

        // Push it on top of the stack
        Self::push(pager, page_id)?;

        Ok(())
    }
    
    /// Pop a free page, and opens it.
    fn pop<P: TraitPager>(pager: &mut P) -> PagerResult<Option<PageId>> 
    {
        if let Some(page_id) = Self::get_free_head(pager)? {
            pager.open_page(&page_id)?;
            Self::set_free_head(pager, FreeHeader::get(pager, &page_id)?.next)?;
            Ok(Some(page_id))
        } else {
            Ok(None)
        }
    }

    fn push<P: TraitPager>(pager: &mut P, page_id: &PageId) -> PagerResult<()> 
    {
        // Set a free header to the page
        FreeHeader{next: Self::get_free_head(pager)?}.set(pager, page_id)?;
        // Define the new free head
        Self::set_free_head(pager, Some(*page_id))?;

        Ok(())
    }

    fn get_free_head<P: TraitPager>(pager: &P) -> PagerResult<Option<PageId>> 
    {
        Ok(PagerHeader::get(pager)?.free_head)
    }

    fn set_free_head<P: TraitPager>(pager: &mut P, free_head: Option<PageId>) -> PagerResult<()> {
        let mut header = PagerHeader::get(pager)?;
        header.free_head = free_head;
        header.set(pager)
    }

}


