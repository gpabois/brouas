use std::fmt::Display;

use crate::io::{DataStream, traits::{InStream, OutStream}};
use super::offset::PageOffset;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct PageSize(u64);

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct BlockSize(u64);

impl Display for BlockSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl BlockSize {
    pub const fn size_of() -> usize {
        std::mem::size_of::<u64>()
    }
}

impl Into<usize> for BlockSize {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl InStream for BlockSize {
    fn read_from_stream<R: std::io::Read>(&mut self, read: &mut R) -> std::io::Result<()> {
        self.0 = DataStream::<u64>::read(read)?;
        Ok(())
    }
}

impl OutStream for BlockSize {
    fn write_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        DataStream::<u64>::write(writer, self.0)
    }

    fn write_all_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        DataStream::<u64>::write_all(writer, self.0)
    }
}

impl From<usize> for BlockSize {
    fn from(v: usize) -> Self {
        Self(v as u64)
    }
}

impl Into<u64> for BlockSize {
    fn into(self) -> u64 {
        self.0
    }
}

impl PageSize {
    pub const fn size_of() -> usize {
        std::mem::size_of::<u64>()
    }
}

impl InStream for PageSize {
    fn read_from_stream<R: std::io::Read>(&mut self, read: &mut R) -> std::io::Result<()> {
        self.0 = DataStream::<u64>::read(read)?;
        Ok(())
    }
}

impl OutStream for PageSize {
    fn write_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        DataStream::<u64>::write(writer, self.0)
    }

    fn write_all_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        DataStream::<u64>::write_all(writer, self.0)
    }
}

impl From<u64> for PageSize {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl Into<u64> for PageSize {
    fn into(self) -> u64 {
        self.0
    }
}

impl Into<usize> for PageSize {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl From<usize> for PageSize {
    fn from(v: usize) -> Self {
        Self(v as u64)
    }
}

impl std::ops::Sub<PageOffset> for PageSize {
    type Output = BlockSize;

    fn sub(self, rhs: PageOffset) -> Self::Output {
        BlockSize(
            self.0.wrapping_sub(rhs.into())
        )
    }
}
