use std::{collections::HashMap};

use super::{id::PageId, page::{Page}};
pub struct PagerBuffer {
    page_size: u64,
    index: HashMap<PageId, Page>,
}

impl PagerBuffer
{
    pub fn new(page_size: u64) -> Self
    {
        Self {
            page_size: page_size,
            index: Default::default()
        }
    }

    pub fn alloc_page<'a>(&'a mut self, page_id: &PageId) -> bool
    {
        if let Some(_) = self.borrow_mut_page(page_id) {
            return true;
        } else {
            unsafe {
                let page = Page::alloc(self.page_size.clone());
                self.index.insert(page_id.clone(), page);
                return true;
            }
        }        
    }

    pub fn drop_page(&mut self, page_id: &PageId) {
        self.index.remove(page_id);
    }

    pub fn borrow_mut_page<'a>(&'a mut self, page_id: &PageId) -> Option<&'a mut Page>
    {
        self.index.get_mut(page_id)
    }

    pub fn borrow_page<'a>(&'a self, page_id: &PageId) -> Option<&'a Page>
    {
        self.index.get(page_id)
    }

}
