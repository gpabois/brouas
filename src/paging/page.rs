use std::ops::Range;

use crate::buffer::BufArray;
use crate::utils::cell::TryCell;
use crate::utils::slice::{Section, BorrowSection, CloneSection, BorrowMutSection, IntoSection};
use crate::utils::borrow::TryBorrowMut;

use self::traits::{ReadPage, Page as TraitPage, WritePage};

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
    pub trait Page {
        type Id;
        type Type;
    }
    pub trait ReadPage : Page {
        fn get_id(&self)        -> Self::Id;
        fn get_type(&self)      -> Self::Type;
        fn get_parent(&self)    -> Self::Id;
        fn get_size(&self)      -> usize;
    }

    pub trait WritePage : Page {
        fn set_id(&mut self, pid: Self::Id);
        fn set_type(&mut self, ptype: Self::Type);
        fn set_parent(&mut self, parent: Self::Id);
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
pub struct Page<'a, Id, Type, Data>(Data, std::marker::PhantomData<&'a ()>);

impl<'a, Id, Type, Data> From<Data> for Page<'a, Id, Type, Data> {
    fn from(data: Data) -> Self {
        Self(data, Default::default())
    }
}
impl<'a, Id, Type, DataCell> TryCell for Page<'a, Id, Type, DataCell> where DataCell: TryBorrowMut<'a, [u8]> {
    type Error = DataCell::Error;
    type Ref = Page<'a, Id, Type, DataCell::Ref>;
    type RefMut = Page<'a, Id, Type, DataCell::RefMut>;
    
    fn try_borrow(&self) -> std::result::Result<Self::Ref, Self::Error> {
        Ok(Page::from(self.0.try_borrow()?))
    }

    fn try_borrow_mut(&mut self) -> std::result::Result<Self::RefMut, Self::Error> {
        Ok(Page::from(self.0.try_borrow_mut()?))
    }
}
impl<'a, Id, Type, DataCell> Page<'a, Id, Type, DataCell> where DataCell: TryBorrowMut<'a, [u8]> {
    pub fn try_new(pid: u64, ptype: u8, data: DataCell) -> std::result::Result<Self, DataCell::Error> {
        let mut pg = Self(data, Default::default());
        
        {
            let mut mpg = pg.try_borrow_mut()?;
            mpg.set_id(pid);
            mpg.set_type(ptype);
        }

        Ok(pg)
    }
}

pub type PageSection<'a, Data> = Section<'a, Data, usize, u8>;
impl<'a, Id, Type, Data> TraitPage for Page<'a, Id, Type, Data> where Data: AsRef<[u8]> {
    type Id = Id;
    type Type = Type;
}

impl<'a, Id, Type, Data> ReadPage for Page<'a, Id, Type, Data> where Data: AsRef<[u8]> {
    fn get_id(&self) -> Id {
        get_id(self.0.as_ref())
    }

    fn get_type(&self) -> Type {
        get_type(self.0.as_ref())
    }

    fn get_parent(&self) -> Id {
        get_parent(self.0.as_ref())
    }

    fn get_size(&self) -> usize {
        self.0.as_ref().len()
    }
}

impl<'a, Id, Type, Data> Page<'a, Id, Type, Data> where Data: AsMut<[u8]> + AsRef<[u8]> {
    fn new(pid: Id, ptype: Type, data: Data) -> Self {
        let mut page = Self(data, Default::default());
        page.set_id(pid);
        page.set_type(ptype);
        page
    }
}
impl<'a, Id, Type, Data> WritePage for Page<'a, Id, Type, Data> where Data: AsMut<[u8]> + AsRef<[u8]> {
    fn set_id(&mut self, pid: Id) {
        set_id(&mut self.0.as_mut(), pid)
    }

    fn set_type(&mut self, ptype: Type) {
        set_type(&mut self.0.as_mut(), ptype)
    }

    fn set_parent(&mut self, parent: Id) {
        set_parent(&mut self.0.as_mut(), parent)
    }
}

impl<'buffer, Id, Type, Data> IntoSection<PageSection<'buffer, Data>> for Page<'buffer, Id, Type, Data> {
    type Cursor = PageSectionType;

    fn into_section(self, cursor: Self::Cursor) -> PageSection<'buffer, Data> {
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
impl<'buffer, Id, Type, Data> CloneSection<PageSection<'buffer, Data>> for Page<'buffer, Id, Type, Data> where Data: Clone {
    type Cursor = PageSectionType;

    fn clone_section(&self, cursor: Self::Cursor) -> PageSection<'buffer, Data> {
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
impl<'buffer, Id, Type, Data> BorrowSection<'buffer, PageSection<'buffer, &'buffer Data>> for Page<'buffer, Id, Type, Data> where Data: AsRef<[u8]> {
    type Cursor = PageSectionType;

    fn borrow_section(&'buffer self, cursor: Self::Cursor) -> PageSection<'buffer, &'buffer Data> {
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
impl<'buffer, Id, Type, Data> BorrowMutSection<'buffer, PageSection<'buffer, &'buffer mut Data>> for Page<'buffer, Id, Type, Data> where Data: AsMut<[u8]> + AsRef<[u8]> {
    type Cursor = PageSectionType;

    fn borrow_mut_section(&'buffer mut self, cursor: Self::Cursor) -> PageSection<'buffer, &'buffer mut Data> {
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

pub type BufPage<'buffer, Id, Type> = Page<'buffer, Id, Type, BufArray<'buffer, u8>>;

impl<'buffer, Id, Type> BufPage<'buffer, Id, Type> {
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

    use crate::{io::Data, fixtures, paging::page::PageSectionType, utils::slice::BorrowMutSection};
    use super::Page;

    #[test]
    pub fn test_page() -> std::io::Result<()> {
        let mut area: [u8; 1000] = [0; 1000];
        let data_size: usize = 100;

        let data = fixtures::random_data(data_size);
        let mut stored_data = Data::with_size(data_size);

        let mut page = Page::new(1u8, 1u8, &mut area[..]);
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