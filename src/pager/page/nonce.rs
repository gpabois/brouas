use std::io::{BufRead, Write};
use rand::Rng;
use std::mem::size_of;

use crate::io::{DataStream, traits::{OutStream, InStream}};

#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct PageNonce(u16);

impl OutStream for PageNonce 
{
    fn write_to_stream<W: Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        DataStream::<u16>::write(writer, self.0)
    }

    fn write_all_to_stream<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        DataStream::<u16>::write_all(writer, self.0)
    }
}

impl InStream for PageNonce {
    fn read_from_stream<B: BufRead>(&mut self, reader: &mut B) -> std::io::Result<()> {
        self.0 = DataStream::<u16>::read(reader)?;
        Ok(())
    }
}

impl PageNonce
{
    pub fn not_set() -> Self {
        PageNonce(0)
    }
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        Self(rng.gen())
    }

    pub const fn size_of() -> usize { size_of::<u16>() }
    
}
