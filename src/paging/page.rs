use std::ops::Range;

use crate::buffer::BufArray;
use crate::utils::cell::TryCell;
use crate::utils::slice::{Section, BorrowSection, CloneSection, BorrowMutSection, IntoSection};
use crate::utils::borrow::{TryBorrowMut};
use crate::utils::borrow::TryBorrow;

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

#[derive(Clone)]
pub struct Page<'a, D>(D, std::marker::PhantomData<&'a ()>);

impl<'a, Data> From<Data> for Page<'a, Data> {
    fn from(data: Data) -> Self {
        Self(data, Default::default())
    }
}

impl<'a, Q> TryCell for Page<'a, Q> where Q: TryBorrowMut<'a, [u8]> {
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

impl<'a, Q> Page<'a, Q> where Q: TryBorrowMut<'a, [u8]> {
    pub fn try_new(pid: u64, ptype: u8, data: Q) -> std::result::Result<Self, Q::Error> {
        let mut pg = Self(data, Default::default());
        
        {
            let mut mpg = pg.try_borrow_mut()?;
            mpg.set_id(pid);
            mpg.set_type(ptype);

        }

        Ok(pg)
    }
}

pub type PageSection<'a, Q> = Section<'a, Q, usize, u8>;

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

impl<'buffer, D> IntoSection<PageSection<'buffer, D>> for Page<'buffer, D>
{
    type Cursor = PageSectionType;

    fn into_section(self, cursor: Self::Cursor) -> PageSection<'buffer, D> {
        match cursor {
            PageSectionType::Body => {
                PageSection::new(self.0, RESERVED..)
            },
            PageSectionType::All => {
                PageSection::new(self.0, ..)
            },
        }
    }
}

impl<'buffer, D> CloneSection<PageSection<'buffer, D>> for Page<'buffer, D> where D: Clone
{
    type Cursor = PageSectionType;

    fn clone_section(&self, cursor: Self::Cursor) -> PageSection<'buffer, D> {
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

impl<'a, Q> BorrowSection<'a, PageSection<'a, &'a Q>> for Page<'a, Q> where Q: AsRef<[u8]>
{
    type Cursor = PageSectionType;

    fn borrow_section(&'a self, cursor: Self::Cursor) -> PageSection<'a, &'a Q> {
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

impl<'a, Q> BorrowMutSection<'a, PageSection<'a, &'a mut Q>> for Page<'a, Q> where Q: AsMut<[u8]> + AsRef<[u8]>
{
    type Cursor = PageSectionType;

    fn borrow_mut_section(&'a mut self, cursor: Self::Cursor) -> PageSection<'a, &'a mut Q> {
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

pub type BufPage<'buffer> = Page<'buffer, BufArray<'buffer, u8>>;

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

    use crate::{io::Data, fixtures, paging::page::PageSectionType, utils::slice::{BorrowSection, BorrowMutSection}};
    use super::Page;

    #[test]
    pub fn test_page() -> std::io::Result<()> {
        let mut area: [u8; 1000] = [0; 1000];
        let data_size: usize = 100;

        let data = fixtures::random_data(data_size);
        let mut stored_data = Data::with_size(data_size);

        let mut page = Page::new(1, 1, &mut area[..]);
        let mut section = page.borrow_mut_section(PageSectionType::Body);
        
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