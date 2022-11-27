use std::marker::PhantomData;

use super::cells::leaf::{LeafCells, VecLeafCells, LeafCell};


pub trait Leaf<const SIZE: usize>
{
    type Hash;
    type Key;
    type Element;
    type Cells: LeafCells<SIZE, Key=Self::Key, Element=Self::Element>;
    
    /// Create a new leaf
    fn new(key: Self::Key, element: Self::Element) -> Self;
    fn from_cells(cells: Self::Cells) -> Self;
    /// Borrow cells
    fn borrow_cells<'a>(&'a self) -> &'a Self::Cells;
    fn borrow_mut_cells<'a>(&'a mut self) -> &'a mut Self::Cells;

    /// Search element behind key
    fn search<'a>(&'a self, key: &Self::Key) -> Option<&'a Self::Element>
    {
        self.borrow_cells().search(key)
    }

    // 
    fn is_full(&self) -> bool
    {
        return self.borrow_cells().is_full();
    }

    // Split the leaf
    fn split_leaf(&mut self) -> (Self::Key, Self) where Self: Sized
    {
        let (key, right_cells) = self.borrow_mut_cells().split();
        (key, Self::from_cells(right_cells))
    }

    // Insert cell
    fn insert(&mut self, key: Self::Key, element: Self::Element)
    {
        self.borrow_mut_cells().insert(key, element);
    }


}

pub struct VecLeaf<const SIZE: usize, Hash, Key: PartialEq + Ord + Clone, Element: Clone>
{
    _h: PhantomData<Hash>,
    cells: VecLeafCells<SIZE, Key, Element>
}

impl<const SIZE: usize, Hash, Key: PartialEq + Ord + Clone, Element: Clone> Leaf<SIZE> for VecLeaf<SIZE, Hash, Key, Element>
{
    type Hash = Hash;
    type Key = Key;
    type Element = Element;
    type Cells = VecLeafCells<SIZE, Key, Element>;

    fn new(key: Self::Key, element: Self::Element) -> Self {
        Self {
            _h: Default::default(),
            cells: VecLeafCells::new(LeafCell::new(key, element))
        }
    }

    fn from_cells(cells: Self::Cells) -> Self {
        Self {
            _h: Default::default(),
            cells: cells
        }
    }

    fn borrow_cells<'a>(&'a self) -> &'a Self::Cells {
        &self.cells
    }

    fn borrow_mut_cells<'a>(&'a mut self) -> &'a mut Self::Cells {
        &mut self.cells
    }
}