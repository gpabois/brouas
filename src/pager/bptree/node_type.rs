use crate::io::{traits::{InStream, OutStream}, DataStream};

pub enum BPTreeNodeType
{
    Unitialised,
    Leaf,
    Branch
}

impl Default for BPTreeNodeType {
    fn default() -> Self {
        Self::Unitialised
    }
}

impl InStream for BPTreeNodeType 
{
    fn read_from_stream<R: std::io::Read>(&mut self, read: &mut R) -> std::io::Result<()> {
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
            BPTreeNodeType::Unitialised => 0,
            BPTreeNodeType::Branch => 1,
            BPTreeNodeType::Leaf => 2
        }
    }
}

impl From<u8> for BPTreeNodeType
{
    fn from(value: u8) -> Self {
        match value {
            0 => BPTreeNodeType::Unitialised,
            1 => BPTreeNodeType::Branch,
            2 => BPTreeNodeType::Leaf,
            _ => panic!("unknown type of b+ tree node")
        }
    }
}

