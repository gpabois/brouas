use super::{page::{PageSize}, PagerResult, header::{PAGE_HEADER_SIZE}};

pub type RawPageOffset = u64;

pub struct PageOffset(u64);

impl From<u64> for PageOffset {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

pub struct PageHeaderOffset(u64);
pub struct PageBodyOffset(pub u64);

pub const PAGE_HEADER_OFFSET: PageHeaderOffset = PageHeaderOffset(0);
pub const PAGE_BODY_OFFSET:   PageBodyOffset = PageBodyOffset(PAGE_HEADER_SIZE);

impl Into<PageOffset> for PageBodyOffset {
    fn into(self) -> PageOffset {
        PageOffset(self.0)
    }
}

impl PageBodyOffset {
    pub const fn const_into(self) -> u64 {
        self.0
    }
}

impl Into<PageOffset> for PageHeaderOffset {
    fn into(self) -> PageOffset {
        PageOffset(self.0)
    }
}

impl PageOffset 
{
    pub unsafe fn new(value: u64) -> Self {
        Self(value)
    }

    pub unsafe fn raw_unchecked(&self) -> RawPageOffset {
        self.0
    }

    pub fn raw(&self, page_size: &PageSize) -> PagerResult<RawPageOffset> {
        unsafe {
            let raw_offset = self.raw_unchecked();
            if raw_offset >= *page_size {
                return Err(super::PagerError::PageOverflow);
            }

            return Ok(raw_offset)
        }
        
    }
}