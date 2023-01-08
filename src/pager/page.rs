use std::{ops::{Deref, DerefMut}, io::Cursor};

/// Page types
pub const ROOT: u8 = 0x1;
pub const BPTREE_LEAF: u8 = 0x2;
pub const BPTREE_BRANCH: u8 = 0x3;

pub const STD_PAGE_SIZE: usize = 16_000;

pub type BrouasPage = Page<STD_PAGE_SIZE>;

pub type PageWriter<'a> = Cursor<&'a mut [u8]>;
pub type PageReader<'a> = Cursor<&'a [u8]>;

pub struct Page<const SIZE: usize>([u8; SIZE]);

impl<const SIZE: usize>  Deref for Page<SIZE>
{
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const SIZE: usize> DerefMut for Page<SIZE>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const SIZE: usize> Page<SIZE>
{
    pub fn new(pid: u64, ptype: u8) -> Self {
        let mut page = Self([0; SIZE]);
        page.set_id(pid);
        page.set_type(ptype);
        page
    }

    pub fn set_id(&mut self, pid: u64) {
        self.0[0..8].copy_from_slice(&pid.to_le_bytes());
    }
    
    pub fn get_id(&self) -> u64 {
        u64::from_le_bytes(self.0[0..3].try_into().unwrap())
    }

    pub fn set_type(&mut self, ptype: u8) {
        self.0[8] = ptype;
    }

    pub fn get_type(&self) -> u64 {
        u64::from_le_bytes(self.0[4..5].try_into().unwrap())
    }

    pub fn set_parent(&mut self, pid: u64) {
        self.0[9..18].copy_from_slice(&pid.to_le_bytes())
    }

    pub fn get_parent(&self) -> u64 {
        u64::from_le_bytes(self.0[9..18].try_into().unwrap())
    }

    pub fn get_writer(&mut self) -> PageWriter<'_> {
        PageWriter::new(&mut self.0[18..])
    }

    pub fn get_reader(&self) -> PageReader<'_> {
        PageReader::new(&self.0[18..])
    }
}


#[cfg(test)]
mod tests {
    use std::io::{Write, Read};

    use crate::{io::Data, fixtures};
    use super::BrouasPage;

    #[test]
    pub fn test_page() -> std::io::Result<()> {
        let data = fixtures::random_data(100);
        let mut stored_data = Data::with_size(100usize);

        let mut page = BrouasPage::new(1, 1);
        
        page.get_writer().write(&data)?;
        page.get_reader().read(&mut stored_data)?;

        assert_eq!(data, stored_data);

        Ok(())
    }
}