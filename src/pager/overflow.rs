use std::{ops::{Deref, DerefMut, Range}, io::{Cursor, Write, BufRead, Read}, cmp::min, borrow::{Borrow, BorrowMut}};

use itertools::Itertools;

use crate::pager::PageId;
use super::{page::{BrouasPage, PageWriter, PageReader, BrouasPageCell}, OVERFLOW_PAGE, Pager};

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

    pub fn get_next(&self) -> PageId {
        u64::from_le_bytes(self.0.deref_body()[OV_NEXT].try_into().unwrap())
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
const AREA_NEXT: Range<usize> = 0..8;
const AREA_IN_SIZE: Range<usize> = 8..10;
const AREA_SIZE: Range<usize> = 10..18;
const AREA_RESERVED: usize = 18;

fn new_overflow_page(pager: &impl crate::pager::traits::Pager) -> crate::pager::Result<OverflowPage<BrouasPageCell>> {
    Ok(
        OverflowPage(
            pager.new_page(OVERFLOW_PAGE)?
        )
    )
}

struct OverlowPageIterator<'a, Pager>(PageId, &'a Pager)
    where Pager: crate::pager::traits::Pager;

impl<'a, Pager> Iterator for OverlowPageIterator<'a, Pager>
where Pager: crate::pager::traits::Pager {
    type Item = OverflowPage<BrouasPageCell>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        } else {
            self.1.get_page(self.0)
            .ok()
            .map(|pg| OverflowPage(pg))
        }
    }
}

fn drop_pages<It>(overflow_pages: &mut It)
where It: Iterator<Item=OverflowPage<BrouasPageCell>> {
    overflow_pages.for_each(|mut opg| opg.drop());
}

/// Write variable data in the area, and overflow pages 
pub fn set_var<Pager>(
    data: &[u8], 
    to: &mut [u8], 
    pager: &Pager
) -> crate::pager::Result<()> 
where Pager: crate::pager::traits::Pager {
    // We can fit everything in the area
    // We only need 8 + 2 bytes
    if data.len() <= to.len() - 2 {
        to[AREA_NEXT].copy_from_slice(&0u64.to_le_bytes());
        to[AREA_IN_SIZE].copy_from_slice(&(data.len() as u16).to_le_bytes());
        let mut to_cursor = Cursor::new(&mut to[10..]);
        to_cursor.write_all(&data)?;
    }
    // We cannot fit everything, so we write overflow pages
    else {
        let next = PageId::from_le_bytes(to[AREA_NEXT].try_into().unwrap());
        let mut data_cursor = Cursor::new(data.deref());
        let in_page_size = (data.len() - AREA_RESERVED) as u16;
        let whole_size: u64 = data.len() as u64;

        let mut opg_it = OverlowPageIterator(next, pager);
        let mut prev: Option<OverflowPage<BrouasPageCell>> = None;
        let mut head: Option<OverflowPage<BrouasPageCell>> = None;
        
        while (data_cursor.position() as usize) < data.len() {
            let mut opg = if let Some(opg) = opg_it.next() {
                opg
            } else {
                new_overflow_page(pager)?
            };

            let written = std::io::copy(
                &mut data_cursor, 
                &mut opg.get_writer()
            )?;

            opg.set_size(written as u16);

            if let Some(mut prev) = prev {
                prev.set_next(opg.get_id())
            }

            if head.is_none() {
                head = Some(opg.clone())
            }

            prev = Some(opg)
        }

        // Drop remaining overflow pages
        drop_pages(&mut opg_it);

        // Write var area header
        to[AREA_NEXT].copy_from_slice(&head.unwrap().get_id().to_le_bytes());
        to[AREA_IN_SIZE].copy_from_slice(&in_page_size.to_le_bytes());
        to[AREA_SIZE].copy_from_slice(&whole_size.to_le_bytes());
    }

    Ok(())
}


pub struct Source<T>(T);

impl<T> Source<T> 
where T: Borrow<[u8]>
{
    pub fn get_next(&self) -> PageId {
        PageId::from_le_bytes(self.0.borrow()[AREA_NEXT].try_into().unwrap())
    }

    pub fn in_size(&self) -> u16 {
        u16::from_le_bytes(self.0.borrow()[AREA_IN_SIZE].try_into().unwrap())       
    }

    pub fn size(&self) -> u64 {
        u64::from_le_bytes(self.0.borrow()[AREA_SIZE].try_into().unwrap())       
    }

    pub fn deref_body(&self) -> &[u8] {
        &self.0.borrow()[AREA_RESERVED..]
    }
}

impl<T> Source<T>
where T: BorrowMut<[u8]> {
    pub fn set_next(&mut self, pid: PageId) {
        self.0.borrow_mut()[AREA_NEXT].copy_from_slice(&pid.to_le_bytes());
    }

    pub fn set_in_size(&mut self, size: u16) {
        self.0.borrow_mut()[AREA_IN_SIZE].copy_from_slice(&size.to_le_bytes());     
    }

    pub fn set_size(&mut self, size: u64) {
        self.0.borrow_mut()[AREA_SIZE].copy_from_slice(&size.to_le_bytes());       
    }

    pub fn deref_mut_body(&mut self) -> &[u8] {
        &mut self.0.borrow_mut()[AREA_RESERVED..]
    }
}

pub enum Section<T> {
    Page(OverflowPage<BrouasPageCell>),
    Source(Source<T>)
}

impl<T> Section<T>
where T: Borrow<[u8]> {
    pub fn get_next(&self) -> PageId {
        match self {
            Section::Page(p) => p.get_next(),
            Section::Source(src) => src.borrow().get_next(),
        }
    }
} 

/// 
pub struct SectionIterator<'a, T, Pager>(Section<T>, &'a Pager)
where Pager: crate::pager::traits::Pager;

impl<'a, T, Pager> Iterator for SectionIterator<'a, T, Pager>
where T: Borrow<[u8]>, Pager: crate::pager::traits::Pager
{
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.get_next() == 0 {
            return None
        }
        
        let page = self.1.get_page(self.0.get_next()).ok()?;
        self.0 = Section::<T>::Page(OverflowPage(page));
        Some(())
    }
}


pub struct VarWriter<'a, Pager>
where Pager: crate::pager::traits::Pager {
    area: &'a mut [u8],
    buffer: [u8; 1000],
    cursor: usize,
    current: State,
    pager: &'a Pager
}

impl<'a, Pager> Write for VarWriter<'a, Pager> 
where Pager: crate::pager::traits::Pager
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.cursor >= self.buffer.len() {
            self.flush()?;
        }

        let rem = self.buffer.len() - self.cursor;
        return Ok(rem)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let buf = &self.buffer[..self.cursor];
    }
}



pub struct VarCursor<'a, Pager> 
where Pager: crate::pager::traits::Pager
{
    position:       usize,
    page_position:  usize,
    whole_size:     usize,
    in_page_size:   usize,
    current:        PageId,
    next:           PageId,
    pager:          &'a Pager
}

impl<'a, Pager> Iterator for VarCursor<'a, Pager>
where Pager: crate::pager::traits::Pager {
    type Item = (PageId, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.position == self.whole_size {
            return None
        }

        if self.page_position <= self.in_page_size {
            let ret = (self.current, self.page_position);
            self.position += 1;
            self.page_position += 1;
            return Some(ret)
        } else {
            if self.next == 0 {
                return None
            }
        let opg = OverflowPage(self.pager.get_page(self.next).ok()?);
            self.current = self.next;
            self.page_position = 0;
            self.next = opg.get_next();

            let ret = (self.current, self.page_position);

            self.position += 1;
            self.page_position = 0;

            return Some(ret)
        }
    }
}

impl<'a, Pager> VarCursor<'a, Pager> 
where Pager: crate::pager::traits::Pager {
    pub fn new(area: &[u8], pager: &'a Pager) -> Self {
        let whole_size = u64::from_le_bytes(area[AREA_SIZE].try_into().unwrap()) as usize;
        let in_page_size = u16::from_le_bytes(area[AREA_IN_SIZE].try_into().unwrap()) as usize;
        let next = u64::from_le_bytes(area[AREA_NEXT].try_into().unwrap());
        let current = 0;

        Self {
            position: 0,
            page_position: 0,
            whole_size,
            in_page_size, 
            current,
            next,
            pager
        }
    }
}

struct VarCursorRanges<'a, Pager>(Vec<(PageId, Range<usize>)>, &'a Pager)
where Pager: crate::pager::traits::Pager;

fn copy_to_buffer<Pager>(area: &[u8], cursor: &mut VarCursor<'_, Pager>, buf: &mut [u8]) -> crate::pager::Result<usize>
where Pager: crate::pager::traits::Pager {
    let ranges = cursor.consume(buf.len());
    let mut cursor = 0usize;

    for (pid, range) in ranges.0.into_iter() {
        match pid {
            0 => {
                let section = &area[AREA_RESERVED..][range];
                let buf_range = cursor..(cursor + section.len());
                buf[buf_range].copy_from_slice(section);
                cursor += section.len()
            },
            pid => {
                let opg = OverflowPage(ranges.1.get_page(pid)?);
                let section = &opg.deref_body()[range];
                let buf_range = cursor..(cursor + section.len());
                buf[buf_range].copy_from_slice(section);
                cursor += section.len()
            }
        };
    }

    Ok(cursor)
}

impl<'a, Pager> VarCursor<'a, Pager> 
where Pager: crate::pager::traits::Pager 
{   
    fn consume(&mut self, len: usize) -> VarCursorRanges<'a, Pager>
    {
        let mut ranges = VarCursorRanges(Default::default(), self.pager);
        for (pid, next) in &self.take(len).into_iter().group_by(|(pid, pos)| *pid) {
            let positions: Vec<_> = next.into_iter().map(|(_, pos)| pos).collect();
            let range = Range::<usize> {
                start: *positions.iter().min().unwrap(), 
                end: *positions.iter().max().unwrap()
            };

            ranges.0.push((pid, range))
        }
        ranges
    }
}

pub struct VarReader<'a, Pager>
where Pager: crate::pager::traits::Pager
{
    area: &'a [u8],
    buffer: [u8; 1000],
    buf_cursor: usize,
    var_cursor: VarCursor<'a, Pager>
}

impl<'a, Pager> VarReader<'a, Pager>
where Pager: crate::pager::traits::Pager
{
    pub fn new(area: &'a [u8], pager: &'a Pager) -> Self {
        Self {
            area,
            buffer: [0; 1000],
            buf_cursor: 0,
            var_cursor: VarCursor::new(area, pager)
        }
    }
}

impl<'a, Pager> Read for VarReader<'a, Pager>
where Pager: crate::pager::traits::Pager {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read = min(buf.len(), self.buffer.len() - self.buf_cursor);
        let range = self.buf_cursor..(self.buf_cursor + read);
        buf[..read].copy_from_slice(&self.buffer[range]);
        Ok(read)
    }
}

impl<'a, Pager> BufRead for VarReader<'a, Pager>
where Pager: crate::pager::traits::Pager {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        copy_to_buffer(self.area, &mut self.var_cursor, &mut self.buffer).expect("Buffer copy error");
        self.buf_cursor = 0;
        Ok(&self.buffer)
    }

    fn consume(&mut self, amt: usize) {
        self.buf_cursor += amt;
    }
}

#[cfg(test)]
mod tests {
    use crate::{io::InMemory, pager::Pager, fixtures};

    #[test]
    fn test_var() -> crate::pager::Result<()>
    {
        let random = fixtures::random_data(100_000);
        let pager = Pager::new(InMemory::new(), 200);
        let mut fixed: [u8; 100] = [0; 100];

        super::set_var(
            &random, 
            &mut fixed, 
            &pager
        )?;

        Ok(())
    }
}
