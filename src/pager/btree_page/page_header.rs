use std::io::{Seek, Write, BufWriter, BufRead};

use crate::pager::page_header::PageHeader;

use super::node_type::BPTreeNodeType;

pub struct BPTreeHeader
{
    /// Type of node: 1 = Leaf, 0 = Branch
    node_type: BPTreeNodeType,
    /// Number of cells
    len: u8,
    /// Capacity of the cells
    capacity: u8
}

impl BPTreeHeader
{
    pub fn seek<S: Seek>(s: &mut S) -> std::io::Result<u64>
    {
        PageHeader::seek_end(s)
    }

    pub fn write_to_buffer<W: Write>(&self, _buffer: &mut BufWriter<W>) -> std::io::Result<usize> {
       todo!()
    }
    
    pub fn read_from_buffer<B: BufRead>(_buffer: &mut B) -> std::io::Result<Self> {
       todo!()
    }
}
