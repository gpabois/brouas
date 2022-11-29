use std::marker::PhantomData;

use crate::tree::cells::leaf::{LeafCells, LeafCell};

use super::cells::leaf::traits::LeafCells as TraitLeafCells;

pub mod traits 
{
    pub trait Leaf
    {
        const SIZE: usize;
        type Hash;
        type Key;
        type Element;
        
        /// Create a new leaf
        fn new(key: Self::Key, element: Self::Element) -> Self;
     
        /// Search element behind key
        fn search<'a>(&'a self, key: &Self::Key) -> Option<&'a Self::Element>;

        // Check if the leaf is full
        fn is_full(&self) -> bool;
    
        // Split the leaf
        fn split_leaf(&mut self) -> (Self::Key, Self) where Self: Sized;
    
        // Insert cell
        fn insert(&mut self, key: Self::Key, element: Self::Element);
    
    }
}


pub struct Leaf<const SIZE: usize, Hash, Key: PartialEq + Ord + Clone, Element: Clone>
{
    _h: PhantomData<Hash>,
    cells: LeafCells<SIZE, Key, Element>
}

impl<const SIZE: usize, Hash, Key: PartialEq + Ord + Clone, Element: Clone> Leaf<SIZE, Hash, Key, Element>
{
    fn from_cells(cells: impl Into<LeafCells<SIZE, Key, Element>>) -> Self {
        Self {
            _h: Default::default(),
            cells: cells.into()
        }
    }
}

impl<const SIZE: usize, Hash, Key, Element> self::traits::Leaf for Leaf<SIZE, Hash, Key, Element>
where Key: PartialEq + Ord + Clone, 
      Element: Clone {
    const SIZE: usize = SIZE;
    type Hash = Hash;
    type Key = Key;
    type Element = Element;

    fn new(key: Self::Key, element: Self::Element) -> Self {
        Self {
            _h: Default::default(),
            cells: LeafCells::new(LeafCell::new(key, element))
        }
    }

    /// Search element behind key
    fn search<'a>(&'a self, key: &Self::Key) -> Option<&'a Self::Element>
    {
        self.cells.search(key)
    }

    // 
    fn is_full(&self) -> bool
    {
        return self.cells.is_full();
    }

    // Split the leaf
    fn split_leaf(&mut self) -> (Self::Key, Self) where Self: Sized
    {
        let (key, right_cells) = self.cells.split();
        (key, Self::from_cells(right_cells))
    }

    // Insert cell
    fn insert(&mut self, key: Self::Key, element: Self::Element)
    {
        self.cells.insert(key, element);
    }
}