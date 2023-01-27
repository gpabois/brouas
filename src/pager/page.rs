use std::{ops::{Range, Deref, DerefMut}};

use crate::{buffer::{BufArray, RefBufArray, RefMutBufArray}, utils::{borrow::{BorrowMut, Borrow}, ops::{GenRange, subrange}, slice::{RefSection, SubSlice, Section}}};

use self::traits::{ReadPage, ReadPageSection, WritePageSection};

use super::{FREE_PAGE};

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
    use std::{ops::{DerefMut, RangeBounds}};

    use crate::utils::borrow::{Borrow, BorrowMut};
    pub trait ReadPageSection {
        type ReadSection: Borrow<[u8]>;

        /// Borrow only the content of the body
        fn borrow_body(&self) -> Self::ReadSection;
        
        /// Borrow the whole content
        fn borrow_all(&self) -> Self::ReadSection;
    }
    pub trait WritePageSection: ReadPageSection {
        type WriteSection: BorrowMut<[u8]>;

        /// Borrow only the content of the body
        fn borrow_mut_body(&mut self) -> Self::WriteSection;

        /// Borrow the whole content
        fn borrow_mut_all(&self) -> Self::WriteSection;
    }
    pub trait ReadPage : ReadPageSection {
        fn get_id(&self) -> u64;
        fn get_type(&self) -> u8;
        fn get_parent(&self) -> u64;
        fn get_size(&self) -> usize;
    }
    pub trait WritePage: WritePageSection {
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

impl<'a, Q> ReadPageSection for Page<'a, Q> 
where Q: Borrow<[u8]> + Clone
{
    type ReadSection = Section<'a, Q, usize, u8>;

    fn borrow_body(&self) -> Self::ReadSection {
        Section::new(self.0.clone(), RESERVED..)
    }

    fn borrow_all(&self) -> Self::ReadSection {
        Section::new(self.0.clone(), ..)
    }
}

impl<'a, Q> ReadPageSection for Page<'a, Q> 
where Q: Borrow<[u8]>
{
    type ReadSection = Section<'a, Q, usize, u8>;

    fn borrow_body(&self) -> Self::ReadSection {
        Section::new(self.0.clone(), RESERVED..)
    }

    fn borrow_all(&self) -> Self::ReadSection {
        Section::new(self.0.clone(), ..)
    }
}


impl<'a, Q> WritePageSection for Page<'a, Q> 
where Q: BorrowMut<[u8]> + Clone
{
    type WriteSection = Section<'a, Q, usize, u8>;

    fn borrow_mut_body(&mut self) -> Self::WriteSection {
        Section::new(self.0.clone(), RESERVED..)
    }

    fn borrow_mut_all(&mut self) -> Self::WriteSection {
        Section::new(self.0.clone(), RESERVED..)
    }
}

#[cfg(test)]
mod tests {
    use crate::{io::Data, fixtures, pager::page::traits::{WritePage}};
    use super::Page;

    #[test]
    pub fn test_page() -> std::io::Result<()> {
        let mut area: [u8; 1000] = [0; 1000];
        let data_size: usize = 100;

        let data = fixtures::random_data(data_size);
        let mut stored_data = Data::with_size(data_size);

        let mut page = Page::new(1, 1, &mut area[..]);
        let body = page.borrow_mut_body();

        page.deref_body().read_exact(&mut stored_data)?;

        assert_eq!(data, stored_data);

        Ok(())
    }
}