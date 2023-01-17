use std::{ops::{Deref, DerefMut, Range}, io::{Cursor, Write}};

use crate::pager::PageId;
use super::{page::{BrouasPage, PageWriter, PageReader, BrouasPageCell}, OVERFLOW_PAGE};

pub struct OverflowPage<P>(P)
where P: Deref<Target=BrouasPage> + DerefMut<Target=BrouasPage>;

const SIZE_RANGE: Range<usize> = 0..2;
const NEXT_RANGE: Range<usize> = 2..12;
const RESERVED: usize = 12;

impl<P> OverflowPage<P> 
where P: Deref<Target=BrouasPage> + DerefMut<Target=BrouasPage>
{
    pub fn set_size(&mut self, size: u16) {
        self.0.deref_mut_body()[SIZE_RANGE].copy_from_slice(&size.to_le_bytes());
    }

    pub fn get_size(&mut self) -> u16 {
        u16::from_le_bytes(self.0.deref_body()[SIZE_RANGE].try_into().unwrap())
    }

    pub fn set_next(&mut self, next: PageId) {
        self.0.deref_mut_body()[NEXT_RANGE].copy_from_slice(&next.to_le_bytes());
    }

    pub fn get_next(&self) -> PageId {
        u64::from_le_bytes(self.0.deref_body()[NEXT_RANGE].try_into().unwrap())
    }

    pub fn get_writer(&mut self) -> PageWriter<'_> {
        PageWriter::new(&mut self.0.deref_mut_body()[RESERVED..])
    }

    pub fn get_reader(&self) -> PageReader<'_> {
        PageReader::new(&self.0.deref_body()[RESERVED..])
    }

    pub fn get_id(&self) -> PageId {
        self.0.get_id()
    }
}

/// Ranges for VAR 
const NEXT: Range<usize> = 0..8;
const IN_SIZE_RANGE: Range<usize> = 8..10;
const WHOLE_SIZE_RANGE: Range<usize> = 10..18;
const AREA_RESERVED: usize = 18;

pub fn new_overflow_page(pager: impl crate::pager::traits::Pager) -> crate::pager::Result<OverflowPage<BrouasPageCell>>
{
    Ok(
        OverflowPage(
            pager.new_page(OVERFLOW_PAGE)?
        )
    )
}

/// Write variable data in the area, and overflow pages
pub fn write_var(
    data: impl Deref<Target=[u8]>, 
    mut to: impl DerefMut<Target=[u8]>, 
    pager: impl crate::pager::traits::Pager
) -> crate::pager::Result<()> {
    // We can fit everything in the area
    if data.len() <= to.len() - 2 {
        to[NEXT].copy_from_slice(&0u64.to_le_bytes());
        to[IN_SIZE_RANGE].copy_from_slice(&(data.len() as u16).to_le_bytes());
        let mut to_cursor = Cursor::new(&mut to[10..]);
        to_cursor.write_all(&data)?;
    }
    // We cannot fit everything
    else {
        let mut data_cursor = Cursor::new(&data);
        let in_page_size = (data.len() - AREA_RESERVED) as u16;
        let whole_size: u64 = data.len() as u64;

        let prev: Option<OverflowPage<BrouasPageCell>> = None;
        while (data_cursor.position() as usize) < data.len() {
            let pg = new_overflow_page(pager)?;
            pg.get_writer().write(&mut data_cursor);

            if let Some(prev) = prev {
                prev.set_next(pg.get_id())
            }
        }
    }

    Ok(())
}