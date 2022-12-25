use crate::{pager::{bptree::{node_offset::BPTreeNodeOffset, cell_capacity::BPTreeCellCapacity}, id::PageId, traits::{bptree::BPTreeLeafCell, pager::Pager}, result::PagerResult, versions::v1::V1}, io::{traits::{OutStream, InStream}, DataStream}};


const CELL_HEADER_OFFSET: BPTreeNodeOffset = BPTreeNodeOffset::new(0);
const CELL_ELEMENT_OFFSET: BPTreeNodeOffset = BPTreeNodeOffset::new(CELL_HEADER_OFFSET.0 + BPTreeLeafCellHeader::size_of());

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

impl BPTreeLeafCellHeader 
{
    pub const fn size_of() -> u64 
    {
        PageId::size_of() + 3 * 8
    }
}

impl BPTreeLeafCellHeader 
{
    const SIZE: u64 = Self::size_of();

    fn new(key: u64) -> Self {
        let mut header = Self::default();
        header.index = key;
        return header;
    }

    fn get_key(&self) -> u64 {
        self.index
    }

    fn get_element_size(&self) -> u64 {
        self.size
    }

    fn set_element_size(&mut self, size: u64) {
        self.size = size;
    }

    fn get_in_page_element_size(&self) -> u64 {
        self.in_page_size
    }

    fn set_in_page_element_size(&mut self, in_page_element_size: u64) {
        self.in_page_size = in_page_element_size
    }

    fn get_overflow_page(&self) -> Option<PageId> {
        self.overflow
    }

    fn set_overflow_page(&mut self, page: Option<PageId>) {
        self.overflow = page
    }
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

pub struct BPTreeLeaf;

impl BPTreeLeaf for BPTreeLeaf
{
    fn new<P, C>(pager: &mut P, capacity: BPTreeCellCapacity, cell: &C) -> PagerResult<PageId> 
    where P: Pager<VERSION = V1>, C: BPTreeLeafCell 
    {
        // Create a new BPTreeNode
        let page_id = BPTreeNode::new(pager, BPTreeNodeType::Leaf, capacity.into())?;
        // Insert a cell
        Self::insert(pager, &page_id, cell)?;
        // Page id
        Ok(page_id)
    }
    
    fn find_nearest_cell_by_key<P>(pager: &P, page_id: &PageId, key: u64) -> PagerResult<Option<BPTreeCellId>>
    where P: Pager {
        let node_header = BPTreeNode::read_header(pager, &page_id)?;

        for cell_id in BPTreeCellIndexes::from(&node_header).iter() 
        {
            unsafe {
                let cell_header = Self::read_header_unchecked(pager, page_id, cell_id, &node_header)?;    
                if cell_header.get_key() >= key
                {
                    return Ok(Some(*cell_id));
                }          
            }

        }

        Ok(None)
    }

    fn find_cell_by_key<P>(pager: &P, page_id: &PageId, key: u64) -> PagerResult<Option<BPTreeCellId>>
    where P: Pager {
        let node_header = BPTreeNode::read_header(pager, &page_id)?;

        for cell_id in BPTreeCellIndexes::from(&node_header).iter() 
        {
            unsafe {
                let cell_header = Self::read_header_unchecked(pager, page_id, cell_id, &node_header)?;    
                if cell_header.get_key() == key
                {
                    return Ok(Some(*cell_id));
                }          
            }

        }

        Ok(None)
    }

    fn insert<P, C>(pager: &mut P, page_id: &PageId, cell: &C) -> PagerResult<()> 
    where P: Pager, C: self::traits::BPTreeLeafCell {
        let node_header = BPTreeNode::read_header(pager, &page_id)?;        
        let cell_id = Self::find_nearest_cell_by_key(pager, page_id, cell.get_key().into())?;

        let cell_id = match cell_id {
            Some(cell_id) => cell_id,
            None => BPTreeCellId(node_header.len + 1)
        };

        let mut cell_header = BPTreeLeafCellHeader::default();
        cell_header.index = cell.get_key().into();
        
        unsafe 
        {
            // Insert the cell.
            BPTreeNode::insert_cell(pager, page_id, &cell_id)?;

            // Write the element in the cell.
            Self::write_element_unchecked(
                pager, 
                &page_id, 
                &cell_id,
                &node_header,
                &mut cell_header,
                cell.borrow_element()
            )?;

            // Write the cell header.
            Self::write_header_unchecked(
                pager, 
                &page_id, 
                &cell_id,
                &node_header,
                &cell_header
            )?;
        }

        Ok(())
    }

    fn remove_by_key<P: Pager>(pager: &mut P, page_id: &PageId, key: impl Into<u64>) -> PagerResult<()> {
        match Self::find_cell_by_key(pager, page_id, key.into())? {
            Some(cell_id) => {
                BPTreeNode::remove_cell(pager, page_id, &cell_id)
            },
            None => Ok(())
        }
    }

    fn max_in_page_element_size(size: &BPTreeCellSize) -> u64 
    {
        size.0.wrapping_sub(BPTreeLeafCellHeader::size_of())
    }
}

impl BPTreeLeaf
{
    unsafe fn write_header_unchecked<P>(
        pager: &mut P, 
        page_id: &PageId, 
        cell_id: &BPTreeCellId,
        node_header: &BPTreeNodeHeader,
        cell_header: &BPTreeLeafCellHeader
    ) -> PagerResult<()>
    where P: Pager
    {
        let size = BPTreeCellSize::from(pager.get_page_size(), node_header.capacity);
        
        BPTreeNode::write_cell_unchecked(
            pager, 
            page_id, 
            cell_id, 
            &size, 
            &CELL_HEADER_OFFSET, 
            cell_header
        )?;

        Ok(())
    }

    /// Write element in the cell
    unsafe fn write_element_unchecked<P, E>(pager: &mut P, page_id: &PageId, cell_id: &BPTreeCellId, header: &BPTreeNodeHeader, cell_header: &mut BPTreeLeafCellHeader, element: &E) -> PagerResult<()> 
    where P: Pager, E: OutStream {
        let size = BPTreeCellSize::from(pager.get_page_size(), header.capacity);

        let mut data = DataBuffer::new();
        element.write_to_stream(&mut data)?;

        let in_page_size = Self::max_in_page_element_size(&size);
        let element_size = data.len();
        
        let in_page_data = data.pop_front(in_page_size);
        
        BPTreeNode::write_cell_unchecked(
            pager, 
            page_id, 
            cell_id, 
            &size, 
            &CELL_ELEMENT_OFFSET, 
            &in_page_data
        )?;

        cell_header.in_page_size = in_page_data.len() as u64;
        cell_header.size = element_size as u64;
        cell_header.overflow = Overflow::write(pager, &mut data, cell_header.overflow)?;

        Ok(())
    }

    unsafe fn read_header_unchecked<P>(
        pager: &P, 
        page_id: &PageId, 
        cell_id: &BPTreeCellId, 
        header: &BPTreeNodeHeader
    ) -> PagerResult<impl TraitBPTreeLeafCellHeader> 
    where P: Pager {
        let size = BPTreeCellSize::from(pager.get_page_size(), header.capacity);
        let mut header: BPTreeLeafCellHeader = Default::default();
        
        BPTreeNode::read_cell_unchecked(
            pager, 
            page_id, 
            cell_id, 
            &size, 
            &CELL_HEADER_OFFSET, 
            &mut header
        )?;

        Ok(header)
    }

}

#[cfg(test)]
mod tests {
    use crate::{pager::PagerResult, fixtures};

    use super::BPTREE_LEAF_CELL_V1;

    #[test]
    pub fn test_bptree_leaf() -> PagerResult<()> {
        let pager = fixtures::pager_fixture(4000);

        Ok(())
    }
}