use std::io::{BufRead, Write};

pub struct PageNonce(u16);

impl PageNonce
{
    pub fn raw_size_of() -> u64 { 2 }

    pub fn write_to_buffer<W: Write>(&self, b: &mut std::io::BufWriter<W>) -> std::io::Result<usize>
    {
        b.write(&self.0.to_ne_bytes())
    }
    
    pub fn read_from_buffer<B: BufRead>(buffer: &mut B) -> std::io::Result<Self> {
        let mut id: [u8; 2] = [0; 2];
        buffer.read_exact(&mut id)?;
        Ok(Self(u16::from_ne_bytes(id)))
    }
}
