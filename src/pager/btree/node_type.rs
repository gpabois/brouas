use crate::io::{traits::{InStream, OutStream}, DataStream};

pub enum BPTreeNodeType
{
    Leaf,
    Branch
}

impl InStream for BPTreeNodeType 
{
    fn read_from_stream<R: std::io::BufRead>(&mut self, read: &mut R) -> std::io::Result<()> {
        *self = Self::from(DataStream::<u8>::read(read)?);
        Ok(())
    }
}

impl OutStream for BPTreeNodeType {
    fn write_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        DataStream::<u8>::write(writer, self.into())
    }

    fn write_all_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        DataStream::<u8>::write_all(writer, self.into())
    }
}

impl Into<u8> for &BPTreeNodeType
{
    fn into(self) -> u8 {
        match self {
            BPTreeNodeType::Branch => 0,
            BPTreeNodeType::Leaf => 1
        }
    }
}

impl From<u8> for BPTreeNodeType
{
    fn from(value: u8) -> Self {
        match value {
            0 => BPTreeNodeType::Branch,
            1 => BPTreeNodeType::Leaf,
            _ => panic!("unknown type of b+ tree node")
        }
    }
}

