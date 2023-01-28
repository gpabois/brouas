use std::{ops::{Range}};

use crate::{utils::{borrow::{BorrowMut, Borrow}, slice::{Section}}};

use self::traits::{BorrowPageSection, BorrowMutPageSection};

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
    use std::{ops::{DerefMut}};

    use crate::utils::borrow::{Borrow, BorrowMut};
    pub trait BorrowPageSection {
        type ReadSection: Borrow<[u8]>;

        /// Borrow only the content of the body
        fn borrow_body(&self) -> Self::ReadSection;
        
        /// Borrow the whole content
        fn borrow_all(&self) -> Self::ReadSection;
    }
    pub trait BorrowMutPageSection: BorrowPageSection {
        type WriteSection: BorrowMut<[u8]>;

        /// Borrow only the content of the body
        fn borrow_mut_body(&mut self) -> Self::WriteSection;

        /// Borrow the whole content
        fn borrow_mut_all(&mut self) -> Self::WriteSection;
    }
    pub trait ReadPage : BorrowPageSection {
        fn get_id(&self) -> u64;
        fn get_type(&self) -> u8;
        fn get_parent(&self) -> u64;
        fn get_size(&self) -> usize;
    }
    pub trait WritePage: BorrowMutPageSection {
        type MutBorrowedContent: DerefMut<Target=[u8]>;

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
pub struct Page<'a, Q>(Q, std::marker::PhantomData<&'a ()>);

impl<'a, Q> Page<'a, Q> where Q: BorrowMut<[u8]> {
    pub fn new(pid: u64, ptype: u8, data: Q) -> Self {
        let mut page = Self(data, Default::default());
        page.set_id(pid);
        page.set_type(ptype);
        page
    }

    pub fn set_id(&mut self, pid: u64) {
        set_id(&mut self.0.borrow_mut(), pid)
    }

    pub fn set_type(&mut self, ptype: u8) {
        set_type(&mut self.0.borrow_mut(), ptype)
    }

    pub fn set_parent(&mut self, parent: u64) {
        set_parent(&mut self.0.borrow_mut(), parent)
    }
}



impl<'a, Q> BorrowPageSection for Page<'a, Q> where Q: Borrow<[u8]> + 'a
{
    type ReadSection = Section<'a, &'a Q, usize, u8>;

    fn borrow_body(&self) -> Self::ReadSection {
        Section::new(&self.0, RESERVED..)
    }

    fn borrow_all(&self) -> Self::ReadSection {
        Section::new(&self.0, ..)
    }
}

impl<'a, Q> BorrowMutPageSection for Page<'a, Q> where Q: BorrowMut<[u8]> + 'a
{
    type WriteSection = Section<'a, &'a mut Q, usize, u8>;

    fn borrow_mut_body(&mut self) -> Self::WriteSection {
        Section::new(&mut self.0, RESERVED..)
    }

    fn borrow_mut_all(&mut self) -> Self::WriteSection {
        Section::new(&mut self.0, ..)
    }
}

#[cfg(test)]
mod tests {
    use std::{ops::{DerefMut, Deref}, io::{Write, Read}};

    use crate::{io::Data, fixtures, utils::borrow::{Ref, BorrowMut, Borrow}, pager::page::traits::{BorrowMutPageSection, BorrowPageSection}};
    use super::Page;

    #[test]
    pub fn test_page() -> std::io::Result<()> {
        let mut area: [u8; 1000] = [0; 1000];
        let data_size: usize = 100;

        let data = fixtures::random_data(data_size);
        let mut stored_data = Data::with_size(data_size);

        let mut page = Page::new(1, 1, Ref::from(&mut area[..]));
        page.borrow_mut_body()
        .borrow_mut()
        .deref_mut()
        .write_all(&data);

        page.borrow_body()
        .borrow()
        .deref()
        .read_exact(&mut stored_data)?;

        assert_eq!(data, stored_data);

        Ok(())
    }
}