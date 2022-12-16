use std::io::{BufRead, Write};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum PageType
{
    Collection,
    BTree,
    Overflow
}

impl PageType
{
    pub fn raw_size_of() -> u64 { 1 }

    pub fn write_to_buffer<W: Write>(&self, b: &mut std::io::BufWriter<W>) -> std::io::Result<usize>
    {
        b.write(&[self.into()])
    }

    pub fn read_from_buffer<B: BufRead>(buffer: &mut B) -> std::io::Result<Self> {
        let mut id: [u8; 1] = [0];
        buffer.read_exact(&mut id)?;
        Ok(Self::from(u8::from_ne_bytes(id)))
    }
}


impl Into<u8> for &PageType
{
    fn into(self) -> u8 {
        match self {
            PageType::Collection => 0,
            PageType::BTree => 1,
            PageType::Overflow => 2
        }
    }
}
impl From<u8> for PageType
{
    fn from(value: u8) -> Self {
        match value {
            0 => PageType::Collection,
            1 => PageType::BTree,
            2 => PageType::Overflow,
            _ => panic!("unknown type of page")
        }
    }
}
