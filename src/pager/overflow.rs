use std::{ops::{Deref, DerefMut, Range}, io::{Write, Seek, Read}, cmp::{min, max}, borrow::{Borrow, BorrowMut}};

use crate::pager::PageId;
use super::{page::{PageSection, traits::{WritePage, ReadPage}, PageSectionRef}, OVERFLOW_PAGE};

pub type Result<T> = std::result::Result<T, Error>;
pub enum Error {
    InsufficientSourceSpace{expected: usize, got: usize}
}

const OV_SIZE: Range<usize> = 0..2;
const OV_NEXT: Range<usize> = 2..12;
const OV_RESERVED: usize = 12;

const SOURCE_NEXT: Range<usize> = 0..8;
const SOURCE_IN_SIZE: Range<usize> = 8..10;
const SOURCE_SIZE: Range<usize> = 10..18;
const SOURCE_RESERVED: usize = 18;

#[derive(Clone)]
pub struct OverflowPage<P>(P);

impl<P> OverflowPage<P> where P: WritePage {
    pub fn set_in_size(&mut self, size: u16) {
        self.0.deref_mut_body()[OV_SIZE].copy_from_slice(&size.to_le_bytes());
    }

    pub fn push_in_size_cursor(&mut self, size: usize) {
        let in_size = max(self.get_in_size(), size as u16);
        self.set_in_size(in_size)
    }

    pub fn set_next(&mut self, next: PageId) {
        self.0.deref_mut_body()[OV_NEXT].copy_from_slice(&next.to_le_bytes());
    }

    pub fn deref_mut_body(&mut self) -> &mut [u8] {
        &mut self.0.deref_mut_body()[OV_RESERVED..]
    }

    pub fn drop(&mut self) {
        self.0.borrow_mut().drop()
    }
}
impl<P> OverflowPage<P> where P: ReadPage {
    pub fn get_in_size(&self) -> u16 {
        u16::from_le_bytes(self.0.borrow().deref_body()[OV_SIZE].try_into().unwrap())
    }

    pub fn get_next(&self) -> Option<PageId> {
        let pid = u64::from_le_bytes(self.0.borrow().deref_body()[OV_NEXT].try_into().unwrap());
        if pid == 0 {
            return None
        }
        Some(pid)
    }

    pub fn deref_body(&self) -> &[u8] {
        &self.0.borrow().deref_body()[OV_RESERVED..]
    }

    pub fn get_id(&self) -> PageId {
        self.0.borrow().get_id()
    }
}

fn new_overflow_page<'page, Pager>(pager: &Pager) -> crate::pager::Result<OverflowPage<Pager::Page>> 
where Pager: crate::pager::traits::Pager<'page>
{
    Ok(
        OverflowPage(
            pager.new_page(OVERFLOW_PAGE)?
        )
    )
}

#[derive(Clone)]
pub struct VarSourceRef(PageSectionRef);

impl VarSourceRef {
    fn deref_source<'page, Pager: crate::pager::traits::Pager<'page>>(self, pager: &Pager) -> crate::pager::Result<VarSource<Pager::Page>> {
        Ok(VarSource(self.0.deref_section(pager)?))
    }
}

#[derive(Clone)]
pub struct VarSource<P>(PageSection<P>);

impl<P> TryFrom<PageSection<P>> for VarSource<P> 
where P: ReadPage
{
    type Error = Error;

    fn try_from(value: PageSection<P>) -> Result<Self> {
        if value.len() < SOURCE_RESERVED {
            return Err(Error::InsufficientSourceSpace { expected: SOURCE_RESERVED, got: value.len() })
        }

        Ok(Self(value))
    }
}

impl<P> VarSource<P>
where P: WritePage + ReadPage
{
    pub fn set_size(&mut self, size: u64) {
        self.0.as_mut()[SOURCE_SIZE].copy_from_slice(&size.to_le_bytes())
    }
    
    pub fn set_in_size(&mut self, size: u16) {
        self.0.as_mut()[SOURCE_IN_SIZE].copy_from_slice(&size.to_le_bytes())
    }

    pub fn push_size_cursor(&mut self, size: u64) {
        let size = max(self.get_size(), size as u64);
        self.set_size(size)
    }

    pub fn push_in_size_cursor(&mut self, size: usize) {
        let in_size = max(self.get_in_size(), size as u16);
        self.set_in_size(in_size)
    }

    pub fn set_next(&mut self, pid: PageId) {
        self.0.as_mut()[SOURCE_NEXT].copy_from_slice(&pid.to_le_bytes());
    }

    pub fn deref_mut_body(&mut self) -> &mut [u8] {
        &mut self.0.as_mut()[SOURCE_RESERVED..]
    }
}

impl<P> VarSource<P> 
where P: ReadPage
{
    pub fn get_next(&self) -> Option<PageId> {
        let pid = u64::from_le_bytes(self.0.as_ref()[SOURCE_NEXT].try_into().unwrap());

        if pid == 0 {
            return None
        }

        Some(pid)
    }

    pub fn get_size(&self) -> u64 {
        return u64::from_le_bytes(self.0.as_ref()[SOURCE_SIZE].try_into().unwrap());
    }

    pub fn get_in_size(&self) -> u16 {
        return u16::from_le_bytes(self.0.as_ref()[SOURCE_IN_SIZE].try_into().unwrap());
    }

    pub fn deref_body(&self) -> &[u8] {
        &self.0.as_ref()[SOURCE_RESERVED..]
    }

}

#[derive(Clone)]
pub enum VarSection<P> {
    Overflow(OverflowPage<P>),
    Source(VarSource<P>)
}

impl<P> VarSection<P>
where P: WritePage + ReadPage
{
    pub fn set_next(&mut self, pid: PageId) {
        match self {
            VarSection::Overflow(section) => section.set_next(pid),
            VarSection::Source(src) => src.set_next(pid),
        }
    }

    pub fn push_in_size_cursor(&mut self, cursor: usize) {
        match self {
            VarSection::Overflow(ov) => ov.push_in_size_cursor(cursor),
            VarSection::Source(src) => src.push_in_size_cursor(cursor),
        }
    }
}

impl<P> VarSection<P> 
where P: ReadPage
{
    pub fn get_next(&self) -> Option<PageId> {
        match self {
            VarSection::Overflow(section) => section.get_next(),
            VarSection::Source(src) => src.get_next(),
        }
    }
    
    pub fn get_in_size(&self) -> u16 {
        match self {
            VarSection::Overflow(section) => section.get_in_size(),
            VarSection::Source(src) => src.get_in_size(),
        }        
    }
}

impl<P> VarSection<P>
where P: WritePage + ReadPage
{
    fn deref_mut_body(&mut self) -> &mut [u8] {
        match self {
            VarSection::Overflow(ov) => ov.deref_mut_body(),
            VarSection::Source(src) => src.deref_mut_body(),
        }
    }
}

impl<P> VarSection<P>
where P: ReadPage
{
    fn deref_body(&self) -> &[u8] {
        match self {
            VarSection::Overflow(ov) => ov.deref_body(),
            VarSection::Source(src) => src.deref_body(),
        }
    }
}

pub struct VarSectionIterator<'a, Pager>(VarSourceRef, VarSection<Pager::Page>, &'a Pager)
where Pager: crate::pager::traits::Pager<'a>;

impl<'a, Pager> VarSectionIterator<'a, Pager>
where Pager: crate::pager::traits::Pager<'a>
{
    pub fn new(pager: &'a Pager, source: impl Into<VarSourceRef>) -> Self {
        let src = source.into();

        Self(
            src,
            VarSection::Source(src.clone().deref_source(pager).unwrap()),
            pager
        )
    }

    pub fn borrow_current(&self) -> &VarSection<Pager::Page> {
        &self.1
    }

    pub fn borrow_mut_current(&mut self) -> &mut VarSection<Pager::Page> {
        &mut self.1
    }

    pub fn restart(&mut self) {
        self.1 = VarSection::Source(self.0.clone().deref_source(self.2).unwrap());
    }
}


impl<'a, Pager> Iterator for VarSectionIterator<'a, Pager>
where Pager: crate::pager::traits::Pager<'a>
{
    type Item = VarSection<Pager::Page>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pid) = self.0.get_next() {
            let page = self.2.get_page(pid).ok()?;
            self.1 = VarSection::Overflow(OverflowPage(page));
            return Some(self.1.clone())
        }

        return None
    }
}

pub struct VarStream<'a, Pager>
where Pager: crate::pager::traits::Pager<'a> {
    iterator: VarSectionIterator<'a, Pager>,
    section_cursor: u64,
    var_cursor: u64
}

impl<'page, Pager> VarStream<'page, Pager> 
where Pager: crate::pager::traits::Pager<'page>
{
    pub fn new(pager: &Pager, source: impl Into<VarSource<Pager::Page>>) -> Self {
        Self {
            iterator: VarSectionIterator::new(pager, source),
            section_cursor: 0,
            var_cursor: 0
        }
    }
}

impl<'page, Pager> VarStream<'page, Pager>
where Pager: crate::pager::traits::Pager<'page>, Pager::Page: ReadPage
{
    pub fn get_dest_cursor(&self, pos: std::io::SeekFrom) -> std::io::Result<usize> {
        let dest = match pos {
            std::io::SeekFrom::Start(pos) => pos,
            std::io::SeekFrom::End(pos) => self.iterator.borrow_head().get_size().wrapping_sub(pos as u64),
            std::io::SeekFrom::Current(pos) => {
                if pos >= 0 {
                    self.var_cursor.wrapping_add(pos as u64)
                } else {
                    self.var_cursor.wrapping_sub(pos as u64)
                }
            },
        };

        let eos = self.iterator.borrow_head().get_size();

        if dest > eos {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, format!("max size is : {}", eos)))
        }

        return Ok(dest as usize)
    }

    pub fn walk_to(&mut self, dest: usize) -> std::io::Result<()> {
        self.iterator.restart();

        while self.var_cursor != dest as u64 {
            // In the range of the current page
            if self.var_cursor <= (dest as u64) && (dest as u64) <= self.var_cursor + (self.iterator.borrow_current().get_in_size() as u64) {
                self.section_cursor = (dest as u64) - self.var_cursor;
                self.var_cursor = dest as u64;
            } else {
                let err = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, format!("no more accessible pages"));
                self.var_cursor += self.iterator.borrow_current().get_in_size() as u64;
                self.iterator.next().ok_or(err)?;
            }
        }

        Ok(())
    }
}

impl<'page, Pager> Seek for VarStream<'page, Pager>
where Pager: crate::pager::traits::Pager<'page>
{
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let dest = self.get_dest_cursor(pos)?;
        self.walk_to(dest)?;
        Ok(dest as u64)
    }
}

impl<'page, Pager> Read for VarStream<'page, Pager> 
where Pager: crate::pager::traits::Pager<'page>
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.len() == 0 {
            return Ok(0);
        } 

        let section = self.iterator.borrow_current();
        let rem = section.deref_body().len().wrapping_sub(self.section_cursor as usize);
        let read = min(rem, buf.len());

        let left_buf = &mut buf[..read];
        
        let body_range = (self.section_cursor as usize)..(self.section_cursor as usize) + read;

        left_buf.copy_from_slice(&section.deref_body()[body_range.clone()]);
        drop(left_buf);

        self.var_cursor += read as u64;
        self.section_cursor += read as u64;

        let right_buf = &mut buf[read..];

        // Read the rest in the next page
        if right_buf.len() > 0 && self.iterator.next().is_some()  {
            return Ok(read + self.read(right_buf)?)
        }        

        Ok(read)

    }
}

impl<'page, Pager> Write for VarStream<'page, Pager> 
where Pager: crate::pager::traits::Pager<'page>, Pager::Page: WritePage
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {

        if buf.len() == 0 {
            return Ok(0);
        } 

        let section = self.iterator.borrow_mut_current();

        let rem = section.deref_body().len().wrapping_sub(self.section_cursor as usize);
        let written = min(rem, buf.len());

        let left_buf = &buf[written..];
        let right_buf = &buf[..written];

        let body_range = (self.section_cursor as usize)..(self.section_cursor as usize) + written;
        section.deref_mut_body()[body_range.clone()].copy_from_slice(left_buf);
        section.push_in_size_cursor(body_range.max().unwrap());

        self.var_cursor += written as u64;
        self.section_cursor += written as u64;

        self.iterator.borrow_mut_head().push_size_cursor(self.var_cursor);

        // We have remaining bytes to write
        if right_buf.len() > 0 {
            // No page left, we have to create a new one
            if let None = self.iterator.next() {
                let ov = new_overflow_page(self.iterator.2).map_err(|err|{
                    let io_err: std::io::Error = err.into();
                    io_err
                })?;
                let section = self.iterator.borrow_mut_current();
                section.set_next(ov.get_id());
            }

            // Recursively write the rest
            self.section_cursor = 0;
            return Ok(written + self.write(right_buf)?)
        }

        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    #[test]
    pub fn test_var() -> std::io::Result<()> {
        

        Ok(())
    }
}