use std::{io::{Read, Write, Seek, SeekFrom}, ops::DerefMut};

use crate::{buffer::{Buffer, BufCellIterator}, utils::{Counter, cell::TryCell, slice::{IntoSection, CloneSection}, borrow::TryBorrowMut}};

use self::traits::PageStorage;

use super::{page::{BufPage, PageSectionType, traits::Page, RefBufPage, RefMutPage}, error::Error, result::Result};

pub type PageId = u64;

pub mod traits {
    use crate::paging::page::traits::{Page, ReadPage, WritePage};
    use std::result::Result;

    pub trait PageStorage {
        type Error;

        fn store<Id: AsRef<str>, Data: AsRef<u8>>(&self, id: Id, page: Data) -> std::result::Result<(), Self::Error>;
        fn fetch<Id: AsRef<str>, DataReceiver: AsMut<u8>>(&self, id: Id, data: &mut DataReceiver) -> std::result::Result<(), Self::Error>;
    }

    pub trait Pager<'a> {
        type Error;
        
        type RefPage:       ReadPage;
        type RefMutPage:    WritePage;
        
        /// Create a new page
        fn new_page(&'a self, ptype: <Self::RefPage as Page>::Type) -> Result<<Self::RefPage as Page>::Id, Self::Error>;

        /// Returns an immutable reference to the page
        fn borrow_page(&'a self, pid: &<Self::RefPage as Page>::Id) -> Result<Self::RefPage, Self::Error>;

        /// Returns a mutable reference to the page
        fn borrow_mut_page(&'a self, pid: &<Self::RefMutPage as Page>::Id) -> Result<Self::RefMutPage, Self::Error>;

        /// Drop the page
        fn drop_page(&self, pid: &<Self::RefPage as Page>::Id) -> Result<(), Self::Error>;

        /// Flush upserted pages into the stream
        fn flush(&self) -> Result<(), Self::Error>;
    }
}

pub const PAGE_SIZE: usize = 16_000;
pub const RESERVED: usize = 10;
pub const FREE_PAGE: u8 = 0x00;
pub const OVERFLOW_PAGE: u8 = 0xFF;

pub struct BufPageIterator<'buffer, Page> {
    cells: BufCellIterator<'buffer>,
    pht: std::marker::PhantomData<Page>
}

impl<'buffer, Page> BufPageIterator<'buffer, Page> {
    pub fn new(cells: BufCellIterator<'buffer>) -> Self {
        Self {
            cells,
            pht: Default::default()
        }
    }
}

impl<'buffer, Id, Type> Iterator for BufPageIterator<'buffer, BufPage<'buffer, Id, Type>> 
{
    type Item = BufPage<'buffer, Id, Type>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.cells.next() {
            None => None,
            Some(cell) => cell.try_into_array::<u8>().map(Self::Item::from)
        }
    }
}

pub struct Pager<'buffer, Page, Storage>
where Storage: PageStorage, Page: crate::paging::page::traits::Page, Page::Id: std::ops::AddAssign<u8>
{
    pool: Buffer, 
    store: Storage, 
    counter: Counter<Page::Id>, 
    pht: std::marker::PhantomData<&'buffer ()>
}

pub type BufPager<'buffer, Id, Type, Storage> =  Pager<'buffer, BufPage<'buffer, Id, Type>, Storage>;

impl<'buffer, Id, Type, Storage> self::traits::Pager<'buffer> for BufPager<'buffer, Id, Type, Storage> where Storage: PageStorage, Storage::Error: Into<Error>, Id: std::ops::AddAssign<u8>
{
    type Error = Error;
    type RefPage = RefBufPage<'buffer, Id, Type>;
    type RefMutPage = RefMutPage<'buffer, Id, Type>;

    fn new_page(&'buffer self, ptype: <Self::RefPage as Page>::Type) -> std::result::Result<<Self::RefPage as Page>::Id, Self::Error> {
        let data = self.pool.alloc_array_uninit::<u8>(PAGE_SIZE)?;
        let pid = self.counter.inc();
        let page = BufPage::try_new(pid, ptype, data)?;
        Ok(pid)
    }

    fn borrow_page(&'buffer self, pid: &<Self::RefPage as Page>::Id) -> std::result::Result<Self::RefPage, Self::Error> {
        todo!()
    }

    fn borrow_mut_page(&'buffer self, pid: &<Self::RefMutPage as Page>::Id) -> std::result::Result<Self::RefMutPage, Self::Error> {
        todo!()
    }

    fn drop_page(&self, pid: &<Self::RefPage as Page>::Id) -> std::result::Result<(), Self::Error> {
        todo!()
    }

    fn flush(&self) -> std::result::Result<(), Self::Error> {
        todo!()
    }
}

impl<'buffer, Page, Storage> Pager<'buffer, Page, Storage>
where Storage: PageStorage, Page: crate::paging::page::traits::Page
{
    /// Create a pager
    /// io: The stream to read and write into
    /// buffer_size: number of pages that can be stored in memory
    pub fn new(store: Storage, buffer_size: usize) -> Self {
        Self {
            store,
            pool: Buffer::new_by_array::<u8>(PAGE_SIZE, buffer_size),
            counter: Default::default(),
            pht: Default::default()
        }
    }

    /// Return an iterator over upserted pages.
    pub fn iter_upserted_pages(&self) -> impl Iterator<Item=Page> {
        self.iter().filter(|page| page.is_upserted())
    }

    /// Iterate over in memory pages
    pub fn iter(&self) -> impl Iterator<Item=BufPage> {
        BufPageIterator::new(self.pool.iter())
    }

}

#[cfg(test)]
mod tests {
    use std::{io::{Write, Read}, ops::DerefMut};
    use crate::{io::{InMemory, Data}, fixtures, paging::page::{PageSectionType}, utils::{cell::TryCell, slice::CloneSection, borrow::TryBorrowMut}};
    use super::{traits::Pager, PageId};

    #[test]
    fn test_pager() -> super::Result<()> {
        let pager = super::Pager::new(InMemory::new(), 10);
        
        let data_size: usize = 1000;
        let random = fixtures::random_data(data_size);
        
        let pid: PageId;
        {
            let page = pager.new_page(0x10)?;
            pid = page.try_borrow()?.get_id();
            
            page
            .clone_section(PageSectionType::Body)
            .try_borrow_mut()?
            .deref_mut()
            .write_all(&random)?;
        }

        let mut stored = Data::with_size(data_size);
        let page = pager.get_page(pid)?.try_borrow()?;
        page.clone_section(PageSectionType::Body).as_ref().read(&mut stored)?;
        
        assert_eq!(random, stored);

        Ok(())
    }
}