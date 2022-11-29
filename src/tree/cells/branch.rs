use crate::tree::NodeRef;

use self::traits::BranchCells as TraitBranchCells;

pub mod traits {
    use crate::tree::{NodeRef};

    pub trait BranchCells
    {
        type Key: PartialOrd + PartialEq;
        type Hash: Clone + PartialEq;
        
        /// New branch cells
        fn new(left: NodeRef<Self::Hash>, key: Self::Key, right: NodeRef<Self::Hash>) -> Self;
        /// Search the node based on the key
        fn search<'a>(&'a self, k: &Self::Key) -> &'a NodeRef<Self::Hash>;
        /// Split the cells
        fn split(&mut self) -> (Self::Key, Self);
        /// The cells are full
        fn is_full(&self) -> bool;
        /// Insert a cell
        fn insert(&mut self, left: NodeRef<Self::Hash>, key: Self::Key, right: NodeRef<Self::Hash>);
    }
}


pub struct BranchCells<const SIZE: usize, Key: PartialEq + PartialOrd, Hash: Clone + PartialEq>
{
    head: NodeRef<Hash>,
    cells: Vec<BranchCell<Key, Hash>>
} 

impl<const SIZE: usize, Key: PartialOrd + PartialEq + Clone, Hash: Clone + PartialEq> TraitBranchCells for BranchCells<SIZE, Key, Hash>
{
    type Key = Key;
    type Hash = Hash;
    
    fn search<'a>(&'a self, k: &Self::Key) -> &'a NodeRef<Self::Hash>
    {
        let mut node = &self.head;
        if let Some(n) = self.cells
        .iter()
        .filter(|c| {c <= &k})
        .last().map(|c| &c.1) 
        {
            node = n
        }
        node
    }
 

    fn split(&mut self) -> (Self::Key, Self) {
        let middle_index = SIZE/2;
        let lefts = &self.cells[0..middle_index - 1];
        let rights = &self.cells[middle_index + 1..];
        let middle_cell = self.cells[middle_index].clone();

        let middle_key = middle_cell.0;
        let right_cell = Self {
            head: middle_cell.1,
            cells: rights.iter().cloned().collect()
        };

        self.cells = lefts.iter().cloned().collect();

        return (middle_key, right_cell)
    }

    fn is_full(&self) -> bool {
        self.cells.len() >= SIZE
    }

    fn insert(&mut self, left: NodeRef<Self::Hash>, key: Self::Key, right: NodeRef<Self::Hash>) {
        let (idx, cell) = self.cells
        .iter_mut()
        .enumerate()
        .find(|(_idx, cell)| cell.1 == left)
        .expect("Expecting to find node ref in tree branch");

        let right_key = cell.0.clone();
        cell.0 = key;

        self.cells.insert(idx + 1, BranchCell(right_key, right));
    }

    fn new(left: NodeRef<Self::Hash>, key: Self::Key, right: NodeRef<Self::Hash>) -> Self {
        Self {
            head: left,
            cells: vec![BranchCell(key, right)]
        }
    }
}

#[derive(Clone)]
pub struct BranchCell<K: PartialOrd + PartialEq, H: Clone + PartialEq>(K, NodeRef<H>);

impl<K: PartialOrd + PartialEq, H: Clone + PartialEq> std::cmp::PartialOrd<K> for BranchCell<K, H>
{
    fn partial_cmp(&self, other: &K) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<K: PartialOrd + PartialEq, H: Clone + PartialEq> std::cmp::PartialOrd<&K> for &mut BranchCell<K, H>
{
    fn partial_cmp(&self, other: &&K) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<K: PartialOrd + PartialEq, H: Clone + PartialEq> std::cmp::PartialEq<K> for BranchCell<K, H>
{
    fn eq(&self, other: &K) -> bool {
        self.0 == *other
    }
}
