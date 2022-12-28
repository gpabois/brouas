use std::{collections::HashMap};

use super::{page::{Page, id::PageId}};
pub struct PagerBuffer {
    index: HashMap<PageId, Page>,
}

impl PagerBuffer
{
    pub fn new() -> Self
    {
        Self {
            index: Default::default()
        }
    }

    pub fn add<'a>(&'a mut self, page: Page) 
    {
        self.index.insert(page.get_id(), page);      
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
