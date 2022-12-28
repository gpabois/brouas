use std::{mem::size_of, io::{BufRead, Write}};

use crate::io::{DataStream, traits::{OutStream, InStream}};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PageType
{
    Unitialised,
    Free, 
    Root,
    Collection,
    BTree,
    Overflow,
    Raw,
    Unknown
}

impl Default for PageType {
    fn default() -> Self {
        Self::Unitialised
    }
}

impl OutStream for PageType 
{
    fn write_to_stream<W: Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        DataStream::<u8>::write(writer, self.into())
    }

    fn write_all_to_stream<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        DataStream::<u8>::write_all(writer, self.into())
    }
}

impl InStream for PageType 
{
    fn read_from_stream<R: BufRead>(&mut self, reader: &mut R) -> std::io::Result<()> {
        *self = Self::from(DataStream::<u8>::read(reader)?);
        Ok(())
    }
}

impl PageType
{
    pub const fn size_of() -> usize { size_of::<u8>() }
}


impl Into<u8> for &PageType
{
    fn into(self) -> u8 {
        match self {
            PageType::Unitialised => 0,
            PageType::Free => 1, 
            PageType::Root => 2,
            PageType::Collection => 3,
            PageType::BTree => 4,
            PageType::Overflow => 5,
            PageType::Raw => 6,
            PageType::Unknown => 255
        }
    }
}
impl From<u8> for PageType
{
    fn from(value: u8) -> Self {
        match value {
            0 => PageType::Unitialised,
            1 => PageType::Free,
            2 => PageType::Root,
            3 => PageType::Collection,
            4 => PageType::BTree,
            5 => PageType::Overflow,
            6 => PageType::Raw,
            _ => PageType::Unknown
        }
    }
}
