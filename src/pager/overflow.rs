use std::{ops::{Deref, DerefMut, Range}, borrow::{Borrow, BorrowMut}, io::Write, cmp::min};

use crate::pager::PageId;
use super::{page::{BrouasPage, PageWriter, PageReader, BrouasPageCell, PageSection}, OVERFLOW_PAGE};

#[derive(Clone)]
pub struct OverflowPage<P>(P)
where P: Deref<Target=BrouasPage> + DerefMut<Target=BrouasPage>;

const OV_SIZE: Range<usize> = 0..2;
const OV_NEXT: Range<usize> = 2..12;
const OV_RESERVED: usize = 12;

impl<P> OverflowPage<P> 
where P: Deref<Target=BrouasPage> + DerefMut<Target=BrouasPage> {
    pub fn set_size(&mut self, size: u16) {
        self.0.deref_mut_body()[OV_SIZE].copy_from_slice(&size.to_le_bytes());
    }

    pub fn get_size(&mut self) -> u16 {
        u16::from_le_bytes(self.0.deref_body()[OV_SIZE].try_into().unwrap())
    }

    pub fn set_next(&mut self, next: PageId) {
        self.0.deref_mut_body()[OV_NEXT].copy_from_slice(&next.to_le_bytes());
    }

    pub fn get_next(&self) -> Option<PageId> {
        let pid = u64::from_le_bytes(self.0.deref_body()[OV_NEXT].try_into().unwrap());
        if pid == 0 {
            return None
        }
        Some(pid)
    }

    pub fn get_writer(&mut self) -> PageWriter<'_> {
        PageWriter::new(&mut self.0.deref_mut_body()[OV_RESERVED..])
    }

    pub fn get_reader(&self) -> PageReader<'_> {
        PageReader::new(&self.0.deref_body()[OV_RESERVED..])
    }

    pub fn deref_mut_body(&mut self) -> &mut [u8] {
        &mut self.0.deref_mut_body()[OV_RESERVED..]
    }

    pub fn deref_body(&self) -> &[u8] {
        &self.0.deref_body()[OV_RESERVED..]
    }

    pub fn get_id(&self) -> PageId {
        self.0.get_id()
    }

    pub fn drop(&mut self) {
        self.0.drop()
    }
}

/// Ranges for VAR 
const SOURCE_NEXT: Range<usize> = 0..8;
const SOURCE_IN_SIZE: Range<usize> = 8..10;
const SOURCE_SIZE: Range<usize> = 10..18;
const SOURCE_RESERVED: usize = 18;

fn new_overflow_page(pager: &impl crate::pager::traits::Pager) -> crate::pager::Result<OverflowPage<BrouasPageCell>> {
    Ok(
        OverflowPage(
            pager.new_page(OVERFLOW_PAGE)?
        )
    )
}

#[derive(Clone)]
pub struct VarSource(PageSection);

impl VarSource {
    pub fn get_next(&self) -> Option<PageId> {
        let pid = u64::from_le_bytes(self.0.deref()[SOURCE_NEXT].try_into().unwrap());

        if pid == 0 {
            return None
        }

        Some(pid)
    }

    pub fn set_next(&mut self, pid: PageId) {
        self.0.deref_mut()[SOURCE_NEXT].copy_from_slice(&pid.to_le_bytes());
    }

    pub fn deref_body(&self) -> &[u8] {
        &self.0.deref()[SOURCE_RESERVED..]
    }

    pub fn deref_mut_body(&mut self) -> &mut [u8] {
        &mut self.0.deref_mut()[SOURCE_RESERVED..]
    }
}

#[derive(Clone)]
pub enum VarSection {
    Overflow(OverflowPage<BrouasPageCell>),
    Source(VarSource)
}

impl VarSection {
    pub fn get_next(&self) -> Option<PageId> {
        match self {
            VarSection::Overflow(section) => section.get_next(),
            VarSection::Source(src) => src.get_next(),
        }
    }
    
    pub fn set_next(&mut self, pid: PageId) {
        match self {
            VarSection::Overflow(section) => section.set_next(pid),
            VarSection::Source(src) => src.set_next(pid),
        }
    }
}

impl VarSection {
    fn deref_body(&self) -> &[u8] {
        match self {
            VarSection::Overflow(ov) => ov.deref_body(),
            VarSection::Source(src) => src.deref_body(),
        }
    }

    fn deref_mut_body(&mut self) -> &mut [u8] {
        match self {
            VarSection::Overflow(ov) => ov.deref_mut_body(),
            VarSection::Source(src) => src.deref_mut_body(),
        }
    }
}

pub struct VarSectionIterator<'a, Pager>(VarSection, &'a Pager)
where Pager: crate::pager::traits::Pager;

impl<'a, Pager> Borrow<VarSection> for VarSectionIterator<'a, Pager>
where Pager: crate::pager::traits::Pager {
    fn borrow(&self) -> &VarSection {
        &self.0
    }
}

impl<'a, Pager> BorrowMut<VarSection> for VarSectionIterator<'a, Pager>
where Pager: crate::pager::traits::Pager {
    fn borrow_mut(&mut self) -> &mut VarSection {
        &mut self.0
    }
}

impl<'a, Pager> Iterator for VarSectionIterator<'a, Pager>
where Pager: crate::pager::traits::Pager {
    type Item = VarSection;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pid) = self.0.get_next() {
            self.0 = VarSection::Overflow(OverflowPage(self.1.get_page(pid).ok()?));
            return Some(self.0.clone())
        }

        return None
    }
}

pub struct VarWriter<'a, Pager>
where Pager: crate::pager::traits::Pager {
    iterator: VarSectionIterator<'a, Pager>,
    section_cursor: usize,
    pager: &'a Pager
}

impl<'a, Pager> Write for VarWriter<'a, Pager> 
where Pager: crate::pager::traits::Pager
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let written: usize;
        let left_buf: &[u8];

        {
            let section: &mut VarSection = self.iterator.borrow_mut();

            if buf.len() == 0 {
                return Ok(0);
            } 

            let rem = section.deref_body().len().wrapping_sub(self.section_cursor);
            let written = min(rem, buf.len());
            let left = buf.len() - written;

            let left_buf = &buf[left..];
            let right_buf = &buf[..left];

            let buf_range = 0..written;
            let body_range = self.section_cursor..self.section_cursor + written;
            section.deref_mut_body()[body_range].copy_from_slice(&buf[buf_range]);
        }
        
        if left_buf.len() > 0 {
            if let None = self.iterator.next() {
                let ov = new_overflow_page(self.pager).unwrap();
                let section: &mut VarSection = self.iterator.borrow_mut();
                section.set_next(ov.get_id())
            }
        }

        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
    }
}