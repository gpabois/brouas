use std::io::{Write, Read};

use crate::{io::{traits::{OutStream, InStream}, DataStream}};

use super::{node_type::{BPTreeNodeType}, BPTreeCellCapacity};

#[derive(Default)]
pub struct BPTreeNodeHeader
{
    /// Type of node: 1 = Leaf, 0 = Branch
    pub node_type: BPTreeNodeType,
    /// Number of cells
    pub len: u8,
    /// Capacity of the cells
    pub capacity: u8
}

impl OutStream for BPTreeNodeHeader {
    fn write_to_stream<W: Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        Ok(
            self.node_type.write_to_stream(writer)? +
            DataStream::<u8>::write(writer, self.len)? +
            DataStream::<u8>::write(writer, self.capacity)?
        )
    }

    fn write_all_to_stream<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.node_type.write_all_to_stream(writer)?;
        DataStream::<u8>::write_all(writer, self.len)?;
        DataStream::<u8>::write_all(writer, self.capacity)?;
        Ok(())
    }
}

impl InStream for BPTreeNodeHeader 
{
    fn read_from_stream<R: Read>(&mut self, reader: &mut R) -> std::io::Result<()> {
        self.node_type.read_from_stream(reader)?;
        self.len =  DataStream::<u8>::read(reader)?;
        self.capacity = DataStream::<u8>::read(reader)?;
        Ok(())
    }
}

impl BPTreeNodeHeader
{
    pub fn new(node_type: BPTreeNodeType, capacity: impl Into<BPTreeCellCapacity>) -> Self {
        Self { node_type: node_type, len: 0, capacity: capacity.into() }
    }

    pub  const fn size_of() -> u64 { 24 }
}

pub const BPTREE_HEADER_SIZE: u64 = BPTreeNodeHeader::size_of();

