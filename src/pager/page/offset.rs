use std::mem::size_of;

use crate::io::{traits::{InStream, OutStream}, DataStream};

#[derive(Default, Copy, Clone)]
pub struct PageOffset(u32);

impl PageOffset {
    pub const fn size_of() -> usize {
        size_of::<u32>()
    }
}

impl std::ops::Add<PageOffset> for PageOffset {
    type Output = Self;

    fn add(self, rhs: PageOffset) -> Self::Output {
        Self(self.0.wrapping_add(rhs.0))
    }
}

impl std::ops::Add<usize> for PageOffset {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0.wrapping_add(rhs as u32))
    }
}

impl InStream for PageOffset {
    fn read_from_stream<R: std::io::BufRead>(&mut self, read: &mut R) -> std::io::Result<()> {
        self.0 = DataStream::<u32>::read(read)?;
        Ok(())
    }
}

impl OutStream for PageOffset {
    fn write_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        DataStream::<u32>::write(writer, self.0)
    }

    fn write_all_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        DataStream::<u32>::write_all(writer, self.0)
    }
}

impl From<usize> for PageOffset {
    fn from(v: usize) -> Self {
        Self(v as u32)
    }
}

impl From<u32> for PageOffset {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<u64> for PageOffset {
    fn from(v: u64) -> Self {
        Self(v as u32)
    }
}

impl Into<u64> for PageOffset {
    fn into(self) -> u64 {
        self.0 as u64
    }
}

impl Into<u64> for &PageOffset {
    fn into(self) -> u64 {
        self.0 as u64
    }
}
