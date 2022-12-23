use crate::{pager::{id::PageId, PagerResult, overflow::{Overflow, OverflowCommand}, page::PageSize, offset::PageOffset}, io::{DataBuffer, traits::{OutStream, InStream}, DataStream}};
use super::{BPTreeNode, node_type::BPTreeNodeType, BPTreeNodeCellCapacity, BP_TREE_BODY_OFFSET};
use crate::pager::traits::Pager;

#[derive(Copy, Clone)]
pub struct BPTreeLeafCellSize(u64);
pub type   BPTreeLeafCellIndex = u8;

impl BPTreeLeafCellSize 
{
    pub const fn from(page_size: PageSize, capacity: BPTreeNodeCellCapacity) -> Self {
        Self(Self::raw_cell_size(page_size, capacity))
    }

    pub fn max_in_page_element_size(&self) -> u64 {
        let cell_size = self.0;
        if cell_size >= BPTreeLeafCellHeader::size_of() {return 0;}
        cell_size - BPTreeLeafCellHeader::size_of()
    }

    const fn raw_cell_size(page_size: PageSize, capacity: BPTreeNodeCellCapacity) -> u64 {
        if BP_TREE_BODY_OFFSET.const_into() >= page_size {return 0;}
        let body_size = page_size - BP_TREE_BODY_OFFSET.const_into();
        let cell_size = body_size / (capacity as u64);
        cell_size    
    }
}

pub struct BPTreeLeafCellOffset(u64);

impl std::ops::Mul<BPTreeLeafCellIndex> for BPTreeLeafCellSize {
    type Output = BPTreeLeafCellOffset;

    fn mul(self, rhs: BPTreeLeafCellIndex) -> Self::Output {
        let offset = (rhs as u64) * self.0;
        BPTreeLeafCellOffset(BP_TREE_BODY_OFFSET.const_into() + offset)
    }
}

impl BPTreeLeafCellOffset 
{
    pub fn to_head(&self) -> BPTreeLeafCellHeadOffset {
        return BPTreeLeafCellHeadOffset(self.0)
    }

    pub fn to_body(&self) -> BPTreeLeafCellBodyOffset {
        self.to_head().to_body()
    }
}

pub struct BPTreeLeafCellHeadOffset(u64);

impl Into<PageOffset> for BPTreeLeafCellHeadOffset {
    fn into(self) -> PageOffset {
        unsafe {
            PageOffset::new(self.0)           
        }
    }
}

impl BPTreeLeafCellHeadOffset {
    // Returns the offset to the overflow field in the cell header.
    pub fn to_overflow_field(&self) -> PageOffset {
        unsafe {
            PageOffset::new(self.0 + 64 * 3)          
        }
    }
    // To body cell
    pub fn to_body(&self) -> BPTreeLeafCellBodyOffset 
    {
        BPTreeLeafCellBodyOffset(self.0 + BPTreeLeafCellHeader::size_of())
    }
}

pub struct BPTreeLeafCellBodyOffset(u64);

impl Into<PageOffset> for BPTreeLeafCellBodyOffset {
    fn into(self) -> PageOffset {
        unsafe {
            PageOffset::new(self.0)          
        }
    }
}

/// A tree leaf cell
/// Header: 256 bytes per cell
/// Payload: Page size - 256 - Page header size
/// If > payload: throw the remaining into overflow pages
#[derive(Default)]
pub struct BPTreeLeafCellHeader
{
    /// The element index
    pub index: u64,
    /// The total element size, including overflow
    pub size: u64,
    /// The portion stored on the current page
    pub in_page_size:   u64,
    /// Pointer to the overflow page
    pub overflow: Option<PageId>
}

impl OutStream for BPTreeLeafCellHeader {
    fn write_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        Ok(
            DataStream::<u64>::write(writer, self.index)? +
            DataStream::<u64>::write(writer, self.size)? +
            DataStream::<u64>::write(writer, self.in_page_size)? +
            self.overflow.write_to_stream(writer)?
        )
    }

    fn write_all_to_stream<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> 
    {
        DataStream::<u64>::write_all(writer, self.index)?;
        DataStream::<u64>::write_all(writer, self.size)?;
        DataStream::<u64>::write_all(writer, self.in_page_size)?;
        self.overflow.write_all_to_stream(writer)?;
    
        Ok(())
    }
}

impl InStream for BPTreeLeafCellHeader {

    fn read_from_stream<R: std::io::BufRead>(&mut self, reader: &mut R) -> std::io::Result<()> {
        self.index = DataStream::<u64>::read(reader)?;
        self.size = DataStream::<u64>::read(reader)?;
        self.in_page_size = DataStream::<u64>::read(reader)?;
        self.overflow.read_from_stream(reader)?;
        Ok(())
    }
}

impl BPTreeLeafCellHeader {
    pub const fn size_of() -> u64 {
        PageId::size_of() + 3 * 8
    }
}

pub mod traits {
    use crate::io::traits::OutStream;

    pub trait BPTreeLeafCell {
        type Key: Into<u64>;
        type Element: OutStream;

        fn borrow_element(&self) -> &Self::Element;
        fn get_key(&self) -> Self::Key;
    }
}

pub struct BPTreeLeaf();

impl BPTreeLeaf
{
    unsafe fn write_cell_header<P>(
        pager: &mut P, 
        page_id: &PageId, 
        offset: BPTreeLeafCellHeadOffset, 
        header: BPTreeLeafCellHeader
    ) -> PagerResult<()>
    where P: Pager
    {
        pager.write_all_to_page(page_id, &header, offset)
    }

    /// Write element in the cell
    /// 
    unsafe fn write_cell_element<P, E>(
        pager: &mut P, 
        page_id: PageId, 
        element: &E,
        size: BPTreeLeafCellSize, 
        index: BPTreeLeafCellIndex, 
        overflow: &mut Overflow
    ) -> PagerResult<()> 
    where P: Pager, E: OutStream
    {
        let mut data = DataBuffer::new();
        element.write_to_stream(&mut data)?;

        let in_page_size = size.max_in_page_element_size();        
        let cell_offset = size * index;
        let element_size = data.len();
        
        let in_page_data = data.pop_front(in_page_size);
        pager.write_all_to_page(&page_id, &data, cell_offset.to_body())?; 

        // Update the header accordingly
        let mut header = pager.read_and_instantiate_from_page::<BPTreeLeafCellHeader, _>(&page_id, cell_offset.to_head())?;
        header.in_page_size = in_page_data.len() as u64;
        header.size = element_size  as u64;

        todo!();
        //overflow.write(OverflowCommand::new(page_id, cell_offset.to_head().to_overflow_field(), data, header.overflow))
        Ok(())
    }

    pub fn new<P, C>(cell: &C, pager: &mut P, capacity: BPTreeNodeCellCapacity, overflow: &mut Overflow) -> PagerResult<PageId> 
    where P: Pager, C: self::traits::BPTreeLeafCell
    {
        // Create a new BPTreeNode
        let page_id = BPTreeNode::new(pager, BPTreeNodeType::Leaf, capacity.into())?;

        let cell_size = BPTreeLeafCellSize::from(pager.get_page_size(), capacity);
        let cell_offset = cell_size * 0;

        unsafe {
            // Write cell header
            let mut header = BPTreeLeafCellHeader::default();
            header.index = cell.get_key().into();
            Self::write_cell_header(pager, &page_id, cell_offset.to_head(), header)?;

            // Write the element
            Self::write_cell_element(pager, page_id, cell.borrow_element(), cell_size, 0, overflow)?;
        }

        Ok(page_id)
    }
}
