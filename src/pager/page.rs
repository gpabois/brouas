use std::{ops::{Deref, DerefMut, Range}, io::Read};

use crate::buffer::{BufArray, RefBufArray, RefMutBufArray};

use self::traits::{WritePage, ReadPage};

use super::{FREE_PAGE, PageId};

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
    use std::io::Read;

    pub trait ReadPage {
        fn get_id(&self) -> u64;
        fn get_type(&self) -> u8;
        fn get_parent(&self) -> u64;
        fn get_size(&self) -> usize;
        fn deref_body(&self) -> &[u8];
    }

    pub trait WritePage : ReadPage {
        fn set_id(&mut self, pid: u64);
        fn set_type(&mut self, ptype: u8);
        fn set_parent(&mut self, parent: u64);
        fn deref_mut_body(&mut self) -> &mut [u8];
        fn drop(&mut self);
    }
}

pub struct Page<D>(D);

pub type BufPage<'buffer>   = Page<BufArray<'buffer, u8>>;
pub type RefBufPage<'buffer>    = Page<RefBufArray<'buffer, u8>>;
pub type RefMutBufPage<'buffer> = Page<RefMutBufArray<'buffer, u8>>;

impl<D> From<D> for Page<D>
{
    fn from(data: D) -> Self {
        Self(data)
    }
}

impl<'buffer> BufPage<'buffer> {
    pub fn new(pid: u64, ptype: u8, area: BufArray<'buffer, u8>) -> Self {
        let page = Self(area.clone());

        let mut mut_page = page.borrow_mut();
        mut_page.set_id(pid);
        mut_page.set_type(ptype);

        page
    }

    pub fn try_borrow(&self) -> crate::pager::Result<RefBufPage<'buffer>> {
        Ok(RefBufPage::from(self.0.try_borrow()?))
    }

    pub fn borrow(&self) -> RefBufPage<'buffer> {
        self.try_borrow().unwrap()
    }

    pub fn try_borrow_mut(&self) -> crate::pager::Result<RefMutBufPage<'buffer>> {
        Ok(RefMutBufPage::from(self.0.try_borrow_mut()?))
    }

    pub fn borrow_mut(&self) -> RefMutBufPage<'buffer> {
        self.try_borrow_mut().unwrap()
    }
}

impl<'buffer> RefMutBufPage<'buffer> {
    /// Transform a mutable reference into an immutable reference
    pub fn degrade(self) -> RefBufPage<'buffer> {
        RefBufPage::from(self.0.degrade())
    }
}

impl<D> Page<D> {
    pub fn borrow_data(&self) -> &D {
        &self.0
    }

    pub fn borrow_mut_data(&mut self) -> &mut D {
        &mut self.0
    }

    fn into_body(self) -> PageSection<Self> {
        PageSection(self, RESERVED..)
    }
}

impl<D> Deref for Page<D> where D: Deref<Target=[u8]>
{
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.0.deref()
    }
}

impl<D> DerefMut for Page<D> where D: DerefMut<Target=[u8]>
{
    fn deref_mut(&mut self) -> &mut [u8] {
        self.0.deref_mut()   
    }
}

impl<D> Page<D> where D: DerefMut<Target=[u8]> {
    pub fn new(pid: u64, ptype: u8, area: D) -> Self {
        let mut page = Self(area);
        page.set_id(pid);
        page.set_type(ptype);
        page
    }

    pub fn load(area: D) -> Self {
        Self(area)
    }

    pub fn init(&mut self, pid: u64, ptype: u8) {
        self.set_id(pid);
        self.set_type(ptype)
    }
}

impl<D> WritePage for Page<D>
where D: DerefMut<Target=[u8]>
{
    fn set_id(&mut self, pid: u64) {
        self.deref_mut()[ID_RANGE].copy_from_slice(&pid.to_le_bytes());
    }

    fn set_type(&mut self, ptype: u8) {
        self.deref_mut()[TYPE_RANGE].copy_from_slice(&u8::to_le_bytes(ptype))
    }

    fn set_parent(&mut self, pid: u64) {
        self.deref_mut()[PARENT_RANGE].copy_from_slice(&pid.to_le_bytes())
    }

    fn deref_mut_body(&mut self) -> &mut [u8] {
        &mut self.deref_mut()[RESERVED..]
    }

    fn drop(&mut self) {
        self.set_type(FREE_PAGE)
    }
}

impl<D> ReadPage for Page<D>
where D: Deref<Target=[u8]>
{ 
    fn get_id(&self) -> u64 {

        u64::from_le_bytes(
            self.deref()[ID_RANGE]
            .try_into()
            .unwrap()
        )
    }

    fn get_type(&self) -> u8 {
        u8::from_le_bytes(self.deref()[TYPE_RANGE].try_into().unwrap())
    }

    fn get_parent(&self) -> u64 {
        u64::from_le_bytes(self.deref()[PARENT_RANGE].try_into().unwrap())
    }

    fn deref_body(&self) -> &[u8] {
        &self.deref()[RESERVED..]
    }

    fn get_size(&self) -> usize {
        self.deref().len()
    }
}

#[derive(Clone)]
pub struct PageSection<P>(P, Range<usize>);

impl<P> PageSection<P> {
    pub fn new(page: P, range: Range<usize>) -> Self {
        Self(page, range)
    }

    pub fn into_sub(self, range: Range<usize>) -> Self {
        Self::new(self.1, self.1[range])
    }
}


pub type BufCellPageSection<'page> = PageSection<BufPage<'page>>;
pub type RefBufPageSection<'page> = PageSection<RefBufPage<'page>>;
pub type RefMutBufPageSection<'page> = PageSection<RefMutBufPage<'page>>;

impl<'buffer> BufCellPageSection<'buffer> {
    pub fn try_borrow(&self) -> crate::pager::Result<RefBufPageSection<'buffer>> {
        Ok(
            RefBufPageSection::new(self.0.try_borrow()?, self.1.clone())
        )
    }

    pub fn borrow(&self) -> RefBufPageSection<'buffer> {
        self.try_borrow().unwrap()
    }

    pub fn try_borrow_mut(&self) -> crate::pager::Result<RefMutBufPageSection<'buffer>> {
        Ok(
            RefMutBufPageSection::new(self.0.try_borrow_mut()?, self.1.clone())
        )
    }

    pub fn borrow_mut(&self) -> RefMutBufPageSection<'buffer> {
        self.try_borrow_mut().unwrap()
    }
}

impl<P> PageSection<P>
where P: ReadPage
{
    pub fn len(&self) -> usize {
        return self.0.deref_body()[self.1.clone()].len()
    }
}

impl<P> Deref for PageSection<P>
where P: ReadPage
{
    fn deref(&self) -> &[u8] {
        &self.0.deref_body()[self.1.clone()]
    }

    type Target = [u8];
}

impl<P> DerefMut for PageSection<P>
where P: WritePage
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.deref_mut_body()[self.1.clone()]     
    }
}


#[cfg(test)]
mod tests {
    use std::io::{Write, Read};

    use crate::{io::Data, fixtures, pager::page::traits::{WritePage, ReadPage}};
    use super::Page;

    #[test]
    pub fn test_page() -> std::io::Result<()> {
        let mut area: [u8; 1000] = [0; 1000];
        let data_size: usize = 100;

        let data = fixtures::random_data(data_size);
        let mut stored_data = Data::with_size(data_size);

        let mut page = Page::<&mut [u8]>::new(1, 1, &mut area[..]);
        
        page.deref_mut_body().write_all(&data)?;
        page.deref_body().read_exact(&mut stored_data)?;

        assert_eq!(data, stored_data);

        Ok(())
    }
}