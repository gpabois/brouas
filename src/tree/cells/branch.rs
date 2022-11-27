use crate::tree::NodeRef;

pub trait BranchCells<const SIZE: usize>
{
    type Key: PartialOrd + PartialEq;
    type Hash: Clone + PartialEq;

    /// Search the node based on the key
    fn search<'a>(&'a self, k: &Self::Key) -> &'a NodeRef<Self::Hash>;
    /// Split the cells
    fn split(&mut self) -> (Self::Key, Self);
    /// The cells are full
    fn is_full(&self) -> bool;
    /// Insert a cell
    fn insert(&mut self, left: NodeRef<Self::Hash>, key: Self::Key, right: NodeRef<Self::Hash>);
}

pub struct VecBranchCells<const SIZE: usize, Key: PartialEq + PartialOrd, Hash: Clone + PartialEq>
{
    head: NodeRef<Hash>,
    cells: Vec<BranchCell<Key, Hash>>
} 

impl<const SIZE: usize, Key: PartialOrd + PartialEq + Clone, Hash: Clone + PartialEq> BranchCells<SIZE> for VecBranchCells<SIZE, Key, Hash>
{
    type Key = Key;
    type Hash = Hash;
    
    fn search<'a>(&'a self, k: &Self::Key) -> &'a NodeRef<Self::Hash>
    {
        let mut node = &self.head;
        if let Some(n) = self.cells
        .iter()
        .filter(|c| {c <= &k})
        .last().map(|c| &c.0) 
        {
            node = n
        }
        node
    }
 

    fn is_full(&self) -> bool {
        self.cells.len() >= SIZE
    }

    fn split(&mut self) -> (Self::Key, Self) {
        let middle_index = SIZE/2;
        let lefts = &self.cells[0..middle_index - 1];
        let rights = &self.cells[middle_index + 1..];
        let middle_cell = self.cells[middle_index].clone();

        let middle_key = middle_cell.1;
        let right_cell = Self {
            head: middle_cell.0,
            cells: rights.iter().cloned().collect()
        };

        self.cells = lefts.iter().cloned().collect();

        return (middle_key, right_cell)
    }

    fn insert(&mut self, left: NodeRef<Self::Hash>, key: Self::Key, right: NodeRef<Self::Hash>) {
        let (idx, cell) = self.cells
        .iter_mut()
        .enumerate()
        .find(|(_idx, cell)| cell.0 == left)
        .expect("Expecting to find node ref in tree branch");

        let right_key = cell.1.clone();
        cell.1 = key;

        self.cells.insert(idx + 1, BranchCell(right, right_key));
    }
}

#[derive(Clone)]
pub struct BranchCell<K: PartialOrd + PartialEq, H: Clone + PartialEq>(NodeRef<H>, K);

impl<K: PartialOrd + PartialEq, H: Clone + PartialEq> std::cmp::PartialOrd<K> for BranchCell<K, H>
{
    fn partial_cmp(&self, other: &K) -> Option<std::cmp::Ordering> {
        self.1.partial_cmp(other)
    }
}

impl<K: PartialOrd + PartialEq, H: Clone + PartialEq> std::cmp::PartialOrd<&K> for &mut BranchCell<K, H>
{
    fn partial_cmp(&self, other: &&K) -> Option<std::cmp::Ordering> {
        self.1.partial_cmp(other)
    }
}


impl<K: PartialOrd + PartialEq, H: Clone + PartialEq> std::cmp::PartialEq<K> for BranchCell<K, H>
{
    fn eq(&self, other: &K) -> bool {
        self.1 == *other
    }
}
