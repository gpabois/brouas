use crate::{pager::{id::PageId, page_type::PageType, PagerResult, change_page_type}, io::traits::{InStream, OutStream}};

use super::{page::PAGE_BODY_OFFSET, pager::header::PagerHeader, V1};
use crate::pager::traits::pager::Pager;
use crate::pager::traits::allocator::Allocator as TraitAllocator;

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
    pub fn get<P: Pager>(pager: &P, page_id: &PageId) -> PagerResult<Self> {
        pager.assert_page_type(page_id, &PageType::Free)?;
        unsafe {
            Self::get_unchecked(pager, page_id)
        }
    }

    pub fn set<P: Pager>(&self, pager: &mut P, page_id: &PageId) -> PagerResult<()> {
        pager.assert_page_type(page_id, &PageType::Free)?;
        unsafe {
            self.set_unchecked(pager, page_id)
        }       
    }

    pub unsafe fn get_unchecked<P: Pager>(pager: &P, page_id: &PageId) -> PagerResult<Self> 
    {
        pager.read_and_instantiate_from_page::<FreeHeader, _>(page_id, PAGE_BODY_OFFSET)
    }

    pub unsafe fn set_unchecked<P: Pager>(&self, pager: &mut P, page_id: &PageId) -> PagerResult<()> 
    {
        pager.write_all_to_page(page_id, self, PAGE_BODY_OFFSET)
    }
}

pub struct Allocator;

impl TraitAllocator for Allocator {
    /// Allocate a new page, if there are any free pages available.
    /// This methods opens the page, if any.
    fn alloc<P>(pager: &mut P) -> PagerResult<Option<PageId>> 
    where P: Pager<VERSION=V1>
    {
        Self::pop(pager)
    }

    /// Free the page, and returns the new head of the free stack list.
    /// This methods does not open the page, nor does it close it or flush it.
    fn free<P: Pager>(pager: &mut P, page_id: &PageId) -> PagerResult<()> 
    where P: Pager<VERSION=V1>
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
    
}

impl Allocator
{   

    /// Pop a free page, and opens it.
    fn pop<P: Pager>(pager: &mut P) -> PagerResult<Option<PageId>> 
    {
        if let Some(page_id) = Self::get_free_head(pager)? {
            pager.open_page(&page_id)?;
            Self::set_free_head(pager, FreeHeader::get(pager, &page_id)?.next)?;
            Ok(Some(page_id))
        } else {
            Ok(None)
        }
    }

    fn push<P: Pager>(pager: &mut P, page_id: &PageId) -> PagerResult<()> 
    {
        // Set a free header to the page
        FreeHeader{next: Self::get_free_head(pager)?}.set(pager, page_id)?;
        // Define the new free head
        Self::set_free_head(pager, Some(*page_id))?;

        Ok(())
    }

    fn get_free_head<P: Pager>(pager: &P) -> PagerResult<Option<PageId>> 
    {
        Ok(PagerHeader::get(pager)?.free_head)
    }

    fn set_free_head<P: Pager>(pager: &mut P, free_head: Option<PageId>) -> PagerResult<()> {
        let mut header = PagerHeader::get(pager)?;
        header.free_head = free_head;
        header.set(pager)
    }

}


