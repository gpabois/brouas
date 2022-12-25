use crate::{io::{traits::{OutStream, InStream}}, pager::page::PageSize};

use self::{node_type::BPTreeNodeType, header::BPTreeNodeHeader};

pub mod leaf;
pub mod node_type;




const BP_TREE_OFFSET: PageOffset = PAGE_BODY_OFFSET;
const BP_TREE_HEADER_OFFSET: PageOffset = BP_TREE_OFFSET;
const BP_TREE_BODY_OFFSET: PageOffset = BP_TREE_HEADER_OFFSET + BPTreeNodeHeader::size_of();

pub type BPTreeNodeV1 = BPTreeNode;
pub struct BPTreeNode;

impl self::traits::BPTreeNode for BPTreeNode {
    fn new<P>(pager: &mut P, node_type: BPTreeNodeType, capacity: u8) -> PagerResult<PageId> 
    where P: Pager {
        let page_id = pager.new_page(PageType::BTree)?;
        let header = BPTreeNodeHeader::new(node_type, capacity);
        
        unsafe {
            Self::write_header_unchecked(pager, &page_id, &header)?;
        }
        
        Ok(page_id)
    } 
    
    /// Insert a cell at the given index, shift any nodes in the process.
    fn insert_cell<P: Pager>(pager: &mut P, page_id: &PageId, cell_id: &BPTreeCellId) -> PagerResult<()> {
        let mut header = Self::read_header(pager, page_id)?;
        
        if cell_id.0 > header.len {
            return Err(super::PagerError::SparseCell)
        }

        unsafe {
            Self::insert_cell_unchecked(pager, page_id, cell_id, &mut header)
        }
    }

    fn remove_cell<P: Pager>(pager: &mut P, page_id: &PageId, cell_id: &BPTreeCellId) -> PagerResult<()> {
        let mut header = Self::read_header(pager, page_id)?;

        if cell_id.0 >= header.len {
            return Err(super::PagerError::OutOfBoundCell);
        }   

        unsafe {
            Self::remove_cell_unchecked(pager, page_id, cell_id, &mut header)
        }
    }
    fn write_cell<P: Pager, D: OutStream>(       
        pager:       &mut P, 
        page_id:     &PageId, 
        cell_id:     &BPTreeCellId,
        cell_offset: &BPTreeNodeOffset,
        cell:        &D) -> PagerResult<()>
    {
        let header = Self::read_header(pager, page_id)?;
        let cell_size = BPTreeCellSize::from(pager.get_page_size(), header.capacity);
        
        if cell_id.0 >= header.len {
            return Err(super::PagerError::OutOfBoundCell);
        }
        
        unsafe {
            Self::write_cell_unchecked(pager, page_id, cell_id, &cell_size, cell_offset, cell)
        }
    }


    fn read_cell<P: Pager, D: InStream>(
        pager:       &P, 
        page_id:     &PageId, 
        cell_id:     &BPTreeCellId,
        cell_offset: &BPTreeNodeOffset,
        cell:        &mut D
    ) -> PagerResult<()> {
        let header = Self::read_header(pager, page_id)?;
        let cell_size = BPTreeCellSize::from(pager.get_page_size(), header.capacity);
        
        if cell_id.0 >= header.len {
            return Err(super::PagerError::OutOfBoundCell);
        }   

        unsafe {
            Self::read_cell_unchecked(pager, page_id, cell_id, &cell_size, cell_offset, cell)
        }
    }

}

impl BPTreeNode 
{
    fn write_header<P>(pager: &mut P, page_id: &PageId, header: &BPTreeNodeHeader) -> PagerResult<()> 
    where P: Pager
    {
        pager.assert_page_type(page_id, &PageType::BTree)?;
        unsafe
        {

            Self::write_header_unchecked(pager, &page_id, header)
        }
    }

    pub unsafe fn write_header_unchecked<P>(pager: &mut P, page_id: &PageId, header: &BPTreeNodeHeader) -> PagerResult<()> 
    where P: Pager
    {
        pager.write_all_to_page(&page_id, header, BP_TREE_HEADER_OFFSET)
    }

    pub unsafe fn read_header_unchecked<P>(pager: &P, page_id: &PageId) -> PagerResult<BPTreeNodeHeader> 
    where P: Pager{
        unsafe {
            pager.read_and_instantiate_from_page::<BPTreeNodeHeader, _>(page_id, BP_TREE_HEADER_OFFSET)
        }
    }

    fn left_shift_cells<P>(
        pager: &mut P, 
        page_id: &PageId,
        cell_id: &BPTreeCellId,
        header: &BPTreeNodeHeader        
    ) -> PagerResult<()> 
    where P: Pager {
        let cells: BPTreeCellIndexes = header.into();
        let cell_size = BPTreeCellSize::from(pager.get_page_size(), header.capacity);
        
        // Get the indexes to shift
        let (_, to_shift_indexes) = cells.0.split_at(cell_id.0 as usize);

        // Nothing to shift, that's good 
        if to_shift_indexes.is_empty() 
        {
            return Ok(());
        }

        // Determine the raw data section to shift
        let head: PageOffset = (cell_size * *to_shift_indexes.first().unwrap()).into();
        let tail: PageOffset = (cell_size * (*to_shift_indexes.last().unwrap() + 1)).into();
        let base: PageOffset = (cell_size * (*cell_id - 1)).into();
        
        let size = tail.wrapping_sub(head);

        unsafe {
            move_page_section(
                pager, 
                page_id, 
                &head, 
                page_id, 
                &base, 
                size as usize
            )?;
        } 
        
        Ok(())
    }
    
    // Shift cells to the rights from the given index.
    fn right_shift_cells<P>(
        pager: &mut P, 
        page_id: &PageId,
        cell_id: &BPTreeCellId,
        header: &BPTreeNodeHeader
    ) -> PagerResult<()> 
    where P: Pager
    {
        let cells: BPTreeCellIndexes = header.into();
        let cell_size = BPTreeCellSize::from(pager.get_page_size(), header.capacity);
        
        // Get the indexes to shift
        let (_, to_shift_indexes) = cells.0.split_at(cell_id.0 as usize);

        // Nothing to shift, that's good 
        if to_shift_indexes.is_empty() 
        {
            return Ok(());
        }

        // Determine the raw data section to shift
        let head: PageOffset = (cell_size * *to_shift_indexes.first().unwrap()).into();
        let tail: PageOffset = (cell_size * (*to_shift_indexes.last().unwrap() + 1)).into();
        let base: PageOffset = (cell_size * (*cell_id + 1)).into();
        
        let size = tail.wrapping_sub(head);

        unsafe {
            move_page_section(
                pager, 
                page_id, 
                &head, 
                page_id, 
                &base, 
                size as usize
            )?;
        }

        Ok(())
    }

    /// Write the cell at the given index.
    /// This method does not check if the given index will create an invalid node (sparse cell).
    pub unsafe fn write_cell_unchecked<P, D: OutStream>(
        pager:       &mut P, 
        page_id:     &PageId, 
        cell_id:     &BPTreeCellId,
        cell_size:   &BPTreeCellSize, 
        cell_offset: &BPTreeNodeOffset,
        cell:        &D
    ) -> PagerResult<()>
    where P: Pager
    {
        let offset: PageOffset = (((*cell_size) * (*cell_id)) + *cell_offset).into();
        
        unsafe {
            pager.write_all_to_page(page_id, cell, offset)
        }
    }


    /// Read the cell at the given index.
    /// This method does not check if the given index will create an invalid node (sparse cell).
    pub unsafe fn read_cell_unchecked<P, D: InStream>(
        pager:       &P, 
        page_id:     &PageId, 
        cell_id:     &BPTreeCellId,
        cell_size:   &BPTreeCellSize, 
        cell_offset: &BPTreeNodeOffset,
        cell:        &mut D
    ) -> PagerResult<()>
    where P: Pager
    {
        let offset: PageOffset = (((*cell_size) * (*cell_id)) + *cell_offset).into();
        
        unsafe {
            pager.read_from_page(cell, page_id, offset)
        }
    }

    pub unsafe fn remove_cell_unchecked<P>(
        pager:      &mut P, 
        page_id:    &PageId, 
        cell_id:    &BPTreeCellId,
        header:     &mut BPTreeNodeHeader
    )  -> PagerResult<()> 
    where P: Pager {

        Self::left_shift_cells(pager, page_id, &(*cell_id + 1), header)?;
        header.len = header.len.wrapping_sub(1u8);
        Self::write_header_unchecked(pager, page_id, header)?;
        Ok(())
    }


    /// Insert a cell at the given index, shift any nodes in the process.
    /// This method does not check if the given index will create an invalid node (sparse cell).
    pub unsafe fn insert_cell_unchecked<P>(
        pager:      &mut P, 
        page_id:    &PageId, 
        cell_id:    &BPTreeCellId,
        header:     &mut BPTreeNodeHeader
    ) -> PagerResult<()> 
    where P: Pager
    {
        // Shift cells, if necessary.
        Self::right_shift_cells(pager, page_id, cell_id, header)?;
        
        header.len = header.len.wrapping_add(1u8);
        Self::write_header_unchecked(pager, page_id, header)?;

        Ok(())
    }

    pub fn read_header<P: Pager>(pager: &P, page_id: &PageId) -> PagerResult<BPTreeNodeHeader> 
    {
        pager.assert_page_type(page_id, &PageType::BTree)?;

        unsafe {
            Self::read_header_unchecked(pager, page_id)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{pager::PagerResult, fixtures::{pager_fixture, random_raw_data}, io::DataBuffer};
    use super::{node_type::BPTreeNodeType, BPTreeNode};
    use super::traits::BPTreeNode as TraitBPTreeNode;

    #[test]
    pub fn test_pager_bptree() -> PagerResult<()> 
    {
        let mut pager = pager_fixture(4000);

        let page_id = BPTreeNode::new(&mut pager, BPTreeNodeType::Leaf, 8)?;

        let cell_1 = random_raw_data(100);
        let cell_2 = random_raw_data(100);
        let mut stored = DataBuffer::with_size(100);

        BPTreeNode::insert_cell(&mut pager, &page_id, &0.into())?;
        BPTreeNode::write_cell(&mut pager, &page_id, &0.into(), &0.into(), &cell_1)?;
        BPTreeNode::read_cell(&mut pager, &page_id, &0.into(), &0.into(), &mut stored)?;
        
        assert_eq!(stored, cell_1);

        BPTreeNode::insert_cell(&mut pager, &page_id, &0.into())?;
        BPTreeNode::write_cell(&mut pager, &page_id, &0.into(), &0.into(), &cell_2)?;
        BPTreeNode::read_cell(&mut pager, &page_id, &0.into(), &0.into(), &mut stored)?;

        assert_eq!(stored, cell_2);

        BPTreeNode::read_cell(&mut pager, &page_id, &1.into(), &0.into(), &mut stored)?;

        assert_eq!(stored, cell_1);
        assert_eq!(BPTreeNode::read_header(&mut pager, &page_id)?.len, 2);

        BPTreeNode::remove_cell(&mut pager, &page_id, &0.into())?;
        BPTreeNode::read_cell(&mut pager, &page_id, &0.into(), &0.into(), &mut stored)?;
        
        assert_eq!(stored, cell_1);
        assert_eq!(BPTreeNode::read_header(&mut pager, &page_id)?.len, 1);

        Ok(())
    }
}