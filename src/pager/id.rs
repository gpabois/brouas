use std::{ops::Add, io::{BufRead, Write}};

use crate::io::{traits::{OutStream, InStream}, DataStream};

use super::page::PageSize;

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone, Default)]
pub struct PageId(u64);

impl std::ops::Mul<PageSize> for PageId {
    type Output = u64;

    fn mul(self, rhs: PageSize) -> Self::Output {
        self.0 * rhs
    }
    
}

impl OutStream for PageId {
    fn write_to_stream<W: Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        DataStream::<u64>::write(writer, self.0)
    }

    fn write_all_to_stream<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        DataStream::<u64>::write_all(writer, self.0)
    }
}

impl InStream for Option<PageId> {
    fn read_from_stream<R: BufRead>(&mut self, read: &mut R) -> std::io::Result<()> 
    {
        let value = DataStream::<u64>::read(read)?;
        if value == 0 {
            *self = None;
        } else {
            *self = Some(PageId(value));
        }

        Ok(())
    }
}

impl OutStream for Option<PageId> {
    fn write_to_stream<W: Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        match self {
            Some(val) => val.write_to_stream(writer),
            None => DataStream::<u64>::write(writer, 0),
        }
    }

    fn write_all_to_stream<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            Some(val) => val.write_all_to_stream(writer),
            None => DataStream::<u64>::write_all(writer, 0),
        }
    }
}

impl InStream for PageId 
{
    fn read_from_stream<R: BufRead>(&mut self, read: &mut R) -> std::io::Result<()> 
    {
        self.0 = DataStream::<u64>::read(read)?;
        Ok(())
    }
}

impl PageId
{
    pub const fn size_of() -> u64 { 8 }
    pub const fn new(val: u64) -> Self {
        Self(val)
    }
}

impl From<u64> for PageId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl Add<u64> for PageId
{
    type Output = PageId;

    fn add(self, rhs: u64) -> Self::Output {
        PageId(self.0 + rhs)
    }
}
