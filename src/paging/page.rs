use std::ops::Range;

use crate::buffer::BufArray;
use crate::utils::cell::TryCell;
use crate::utils::slice::{Section, Sectionable, MutSectionable, TrySectionable, TryMutSectionable};
use crate::utils::borrow::{TryBorrow, TryBorrowMut};

use self::traits::TryReadPage;

/// Page types
pub const ROOT: u8 = 0x1;
pub const BPTREE_LEAF: u8 = 0x2;
pub const BPTREE_BRANCH: u8 = 0x3;

/// Page sections
const ID_RANGE: Range<usize> = 0..8;
const TYPE_RANGE: Range<usize> = 8..9;
const PARENT_RANGE: Range<usize> = 9..18;
const RESERVED: usize = 18;

pub mod traits 
{  
    pub trait ReadPage {
        fn get_id(&self) -> u64;
        fn get_type(&self) -> u8;
        fn get_parent(&self) -> u64;
        fn get_size(&self) -> usize;
    }

    pub trait TryReadPage {
        type Error;

        fn try_get_id(&self) -> std::result::Result<u64, Self::Error>;
        fn try_get_type(&self) -> std::result::Result<u8, Self::Error>;
        fn try_get_parent(&self) -> std::result::Result<u64, Self::Error>;
        fn try_get_size(&self) -> std::result::Result<usize, Self::Error>;
    }

    pub trait WritePage {
        fn set_id(&mut self, pid: u64);
        fn set_type(&mut self, ptype: u8);
        fn set_parent(&mut self, parent: u64);
        fn drop(&mut self);
    }
}

fn set_id(content: &mut [u8], pid: u64) {
    content[ID_RANGE].copy_from_slice(&pid.to_le_bytes());
}
fn set_type(content: &mut [u8], ptype: u8) {
    content[TYPE_RANGE].copy_from_slice(&u8::to_le_bytes(ptype))
}
fn set_parent(content: &mut [u8], parent: u64) {
    content[PARENT_RANGE].copy_from_slice(&parent.to_le_bytes())
}
fn get_id(content: &[u8]) -> u64 {
    u64::from_le_bytes(
        content[ID_RANGE]
        .try_into()
        .unwrap()
    )
}
fn get_type(content: &[u8]) -> u8 {
    u8::from_le_bytes(content[TYPE_RANGE].try_into().unwrap())
}
fn get_parent(content: &[u8]) -> u64 {
    u64::from_le_bytes(content[PARENT_RANGE].try_into().unwrap())
}

pub struct PageCell<'a, D>(D, std::marker::PhantomData<&'a ()>);

impl<'a, DataCell> From<DataCell> for PageCell<'a, DataCell> {
    fn from(data: DataCell) -> Self {
        Self(data, Default::default())
    }
}

impl<'a, Q> TryCell for PageCell<'a, Q> where Q: TryBorrowMut<'a, [u8]>
{
    type Error = Q::Error;
    type Ref = Page<'a, Q::Ref>;
    type RefMut = Page<'a, Q::RefMut>;
    
    fn try_borrow(&self) -> std::result::Result<Self::Ref, Self::Error> {
        Ok(Page::from(self.0.try_borrow()?))
    }

    fn try_borrow_mut(&mut self) -> std::result::Result<Self::RefMut, Self::Error> {
        Ok(Page::from(self.0.try_borrow_mut()?))
    }
}

impl<'a, Q> TryReadPage for PageCell<'a, Q> where Q: TryBorrow<'a, [u8]> {
    type Error = Q::Error;

    fn try_get_id(&self) -> std::result::Result<u64, Self::Error> {
        self.0.try_borrow().map(|page| get_id(page.as_ref()))
    }

    fn try_get_type(&self) -> std::result::Result<u8, Self::Error> {
        self.0.try_borrow().map(|page| get_type(page.as_ref()))
    }

    fn try_get_parent(&self) -> std::result::Result<u64, Self::Error> {
        self.0.try_borrow().map(|page| get_parent(page.as_ref()))

    }

    fn try_get_size(&self) -> std::result::Result<usize, Self::Error> {
        self.0.try_borrow().map(|page| page.as_ref().len())
    }
}

impl<'a, Q> TrySectionable<'a, PageSection<'a, Q::Ref>> for PageCell<'a, Q> where Q: TryBorrow<'a, [u8]> {
    type Cursor = PageSectionType;
    type Error = Q::Error;

    fn try_section(&'a self, cursor: Self::Cursor) -> std::result::Result<PageSection<'a, Q::Ref>, Self::Error> {
        Ok(match cursor {
            PageSectionType::Body => {
                PageSection::new(self.0.try_borrow()?, RESERVED..)
            },
            PageSectionType::All => {
                PageSection::new(self.0.try_borrow()?, ..)
            },
        })
    }
}

impl<'a, Q> TryMutSectionable<'a, PageSection<'a, Q::Ref>> for PageCell<'a, Q> where Q: TryBorrowMut<'a, [u8]> {
    type Cursor = PageSectionType;
    type Error = Q::Error;

    fn try_section_mut(&'a mut self, cursor: Self::Cursor) -> std::result::Result<PageSection<'a, Q::RefMut>, Self::Error> {
        Ok(match cursor {
            PageSectionType::Body => {
                PageSection::new(self.0.try_borrow_mut()?, RESERVED..)
            },
            PageSectionType::All => {
                PageSection::new(self.0.try_borrow_mut()?, ..)
            },
        })
    }
}

impl<'a, Q> PageCell<'a, Q> where Q: TryBorrowMut<'a, [u8]> {
    pub fn try_new(pid: u64, ptype: u8, data: Q) -> std::result::Result<Self, Q::Error> {
        let mut pg = PageCell(data, Default::default());
        
        {
            let mut mpg = pg.try_borrow_mut()?;
            mpg.set_id(pid);
            mpg.set_type(ptype);

        }

        Ok(pg)
    }
}

pub type PageSection<'a, Q> = Section<'a, Q, usize, u8>;

impl<'buffer, D> Sectionable<'buffer, PageSection<'buffer, D>> for PageCell<'buffer, D> 
where D: Clone
{
    type Cursor = PageSectionType;

    fn section(&self, cursor: Self::Cursor) -> PageSection<'buffer, D> {
        match cursor {
            PageSectionType::Body => {
                PageSection::new(self.0.clone(), RESERVED..)
            },
            PageSectionType::All => {
                PageSection::new(self.0.clone(), ..)
            },
        }
    }
}

#[derive(Clone)]
pub struct Page<'a, Q>(Q, std::marker::PhantomData<&'a ()>) where Q: AsRef<[u8]>;

impl<'a, Q> From<Q> for Page<'a, Q> where Q: AsRef<[u8]> {
    fn from(value: Q) -> Self {
        Self(value, Default::default())
    }
}
impl<'a, Q> Page<'a, Q> where Q: AsMut<[u8]> + AsRef<[u8]> {
    pub fn new(pid: u64, ptype: u8, data: Q) -> Self {
        let mut page = Self(data, Default::default());
        page.set_id(pid);
        page.set_type(ptype);
        page
    }

    pub fn set_id(&mut self, pid: u64) {
        set_id(&mut self.0.as_mut(), pid)
    }

    pub fn set_type(&mut self, ptype: u8) {
        set_type(&mut self.0.as_mut(), ptype)
    }

    pub fn set_parent(&mut self, parent: u64) {
        set_parent(&mut self.0.as_mut(), parent)
    }
}
impl<'a, Q> Page<'a, Q> where Q: AsRef<[u8]> {
    pub fn get_id(&self) -> u64 {
        get_id(self.0.as_ref())
    }

    pub fn get_type(&self) -> u8 {
        get_type(self.0.as_ref())
    }

    pub fn get_parent(&self) -> u64 {
        get_parent(self.0.as_ref())
    }

    pub fn get_size(&self) -> usize {
        self.0.as_ref().len()
    }
}

impl<'a, Q> Sectionable<'a, PageSection<'a, &'a Q>> for Page<'a, Q> where Q: AsRef<[u8]>
{
    type Cursor = PageSectionType;

    fn section(&'a self, cursor: Self::Cursor) -> PageSection<'a, &'a Q> {
        match cursor {
            PageSectionType::Body => {
                PageSection::new(&self.0, RESERVED..)
            },
            PageSectionType::All => {
                PageSection::new(&self.0, ..)
            },
        }
    }
}

impl<'a, Q> MutSectionable<'a, PageSection<'a, &'a mut Q>> for Page<'a, Q> where Q: AsMut<[u8]> + AsRef<[u8]>
{
    type Cursor = PageSectionType;

    fn section_mut(&'a mut self, cursor: Self::Cursor) -> PageSection<'a, &'a mut Q> {
        match cursor {
            PageSectionType::Body => {
                PageSection::new(&mut self.0, RESERVED..)
            },
            PageSectionType::All => {
                PageSection::new(&mut self.0, ..)
            },
        }
    }
}

pub type BufPage<'buffer> = PageCell<'buffer, BufArray<'buffer, u8>>;

impl<'buffer> BufPage<'buffer> {
    pub fn is_upserted(&self) -> bool {
        self.0.is_upserted()
    }

    pub fn ack_upsertion(&mut self) {
        self.0.ack_upsertion()
    }
}

pub enum PageSectionType {
    Body,
    All
}

#[cfg(test)]
mod tests {
    use std::io::{Write, Read};

    use crate::{io::Data, fixtures, utils::{slice::{MutSectionable}}, paging::page::PageSectionType};
    use super::Page;

    #[test]
    pub fn test_page() -> std::io::Result<()> {
        let mut area: [u8; 1000] = [0; 1000];
        let data_size: usize = 100;

        let data = fixtures::random_data(data_size);
        let mut stored_data = Data::with_size(data_size);

        let mut page = Page::new(1, 1, &mut area[..]);
        let mut section = page.section_mut(PageSectionType::Body);
        
        section
        .as_mut()
        .write_all(&data)?;

        section
        .as_ref()
        .read_exact(&mut stored_data)?;

        assert_eq!(data, stored_data);

        Ok(())
    }
}