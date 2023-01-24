use std::{ops::{Deref, DerefMut, Range}, io::{Cursor}, borrow::{Borrow, BorrowMut}};

use crate::buffer::BufferCell;

use self::traits::{WritePage, ReadPage};

use super::FREE_PAGE;

/// Page types
pub const ROOT: u8 = 0x1;
pub const BPTREE_LEAF: u8 = 0x2;
pub const BPTREE_BRANCH: u8 = 0x3;

pub type PageWriter<'a> = Cursor<&'a mut [u8]>;
pub type PageReader<'a> = Cursor<&'a [u8]>;

pub mod traits {
    pub trait ReadPage: AsRef<[u8]> {
        fn get_id(&self) -> u64;
        fn get_type(&self) -> u8;
        fn get_parent(&self) -> u64;
        fn get_size(&self) -> usize;
        fn deref_body(&self) -> &[u8];
    }

    pub trait WritePage: AsMut<[u8]> {
        fn set_id(&mut self, pid: u64);
        fn set_type(&mut self, ptype: u8);
        fn set_parent(&mut self, parent: u64);
        fn deref_mut_body(&mut self) -> &mut [u8];
        fn drop(&mut self);
    }
}

pub struct Page<D>(D);

pub type BufPage<'Buffer> = Page<BufferCell<'Buffer, [u8]>>;

impl<D> AsRef<[u8]> for Page<D>
where D: AsRef<[u8]>
{
    fn as_ref(&self) -> &[u8] {
        &self.0.as_ref()
    }
}

impl<D> AsMut<[u8]> for Page<D>
where D: AsMut<[u8]>
{
    fn as_mut(&mut self) -> &mut [u8]{
        &mut self.0.as_mut()
    }
}

const ID_RANGE: Range<usize> = 0..8;
const TYPE_RANGE: Range<usize> = 8..9;
const PARENT_RANGE: Range<usize> = 9..18;
const RESERVED: usize = 18;

impl<D> Page<D>
where D: AsMut<[u8]> {
    pub fn new(pid: u64, ptype: u8, mut area: D) -> Self {
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
where D: AsMut<[u8]> 
{
    fn set_id(&mut self, pid: u64) {
        self.0.as_mut()[ID_RANGE].copy_from_slice(&pid.to_le_bytes());
    }

    fn set_type(&mut self, ptype: u8) {
        self.0.as_mut()[TYPE_RANGE].copy_from_slice(&u8::to_le_bytes(ptype))
    }

    fn set_parent(&mut self, pid: u64) {
        self.0.as_mut()[PARENT_RANGE].copy_from_slice(&pid.to_le_bytes())
    }

    fn deref_mut_body(&mut self) -> &mut [u8] {
        &mut self.0.as_mut()[RESERVED..]
    }

    fn drop(&mut self) {
        self.set_type(FREE_PAGE)
    }
}

impl<D> ReadPage for Page<D>
where D: AsRef<[u8]>
{ 
    fn get_id(&self) -> u64 {
        u64::from_le_bytes(self.0.as_ref()[ID_RANGE].try_into().unwrap())
    }

    fn get_type(&self) -> u8 {
        u8::from_le_bytes(self.0.as_ref()[TYPE_RANGE].try_into().unwrap())
    }

    fn get_parent(&self) -> u64 {
        u64::from_le_bytes(self.0.as_ref()[PARENT_RANGE].try_into().unwrap())
    }

    fn deref_body(&self) -> &[u8] {
        &self.0.as_ref()[RESERVED..]
    }

    fn get_size(&self) -> usize {
        self.0.as_ref().len()
    }
}

#[derive(Clone)]
pub struct PageSection<B, P>(B, Range<usize>, std::marker::PhantomData<P>);

impl<B, P> PageSection<B, P>
where B: AsRef<P>, P: ReadPage
{
    pub fn len(&self) -> usize {
        return self.0.as_ref().deref_body()[self.1.clone()].len()
    }
}

impl<B, P> AsRef<[u8]> for PageSection<B, P>
where B: AsRef<P>, P: ReadPage
{
    fn as_ref(&self) -> &[u8] {
        &self.0.as_ref().deref_body()[self.1.clone()]
    }
}

impl<B, P> AsMut<[u8]> for PageSection<B, P>
where B: AsMut<P>, P: WritePage
{
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0.as_mut().deref_mut_body()[self.1.clone()]
    }
}


#[cfg(test)]
mod tests {
    use std::io::{Write, Read};

    use crate::{io::Data, fixtures};
    use super::Page;

    #[test]
    pub fn test_page() -> std::io::Result<()> {
        let area: [u8; 100] = [0;100];
        let data = fixtures::random_data(100);
        let mut stored_data = Data::with_size(100usize);

        let mut page = Page::new(1, 1, &mut area);
        
        /*
        page.get_writer().write(&data)?;
        page.get_reader().read(&mut stored_data)?;
        */

        assert_eq!(data, stored_data);

        Ok(())
    }
}