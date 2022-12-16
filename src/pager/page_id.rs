use std::{ops::Add, io::{BufRead, Write, BufWriter}};

#[derive(Hash, PartialEq, Eq, Copy, Clone, Default)]
pub struct PageId(u64);

impl PageId
{
    pub fn raw_size_of() -> u64 { 8 }

    pub fn write_to_buffer<W: Write>(&self, b: &mut BufWriter<W>) -> std::io::Result<usize>
    {
        b.write(&self.0.to_ne_bytes())
    }
    
    pub fn read_from_buffer<B: BufRead>(buffer: &mut B) -> std::io::Result<Self> {
        let mut id: [u8; 8] = [0; 8];
        buffer.read_exact(&mut id)?;
        Ok(Self(u64::from_ne_bytes(id)))
    }
}

impl Add<u64> for PageId
{
    type Output = PageId;

    fn add(self, rhs: u64) -> Self::Output {
        PageId(self.0 + rhs)
    }
}
