use super::{node::{BPTreeNodeId}, nodes::traits::{BPTreeNodes, Split}};

struct BranchCell<Key>
{
    left: BPTreeNodeId,
    key: Key,
}

impl<K> From<(BPTreeNodeId, K)> for BranchCell<K> {
    fn from(value: (BPTreeNodeId, K)) -> Self {
        Self { left: value.0, key: value.1 }
    }
}

impl <K> Into<(BPTreeNodeId, K)> for BranchCell<K> {
    fn into(self) -> (BPTreeNodeId, K) {
        (self.left, self.key)
    }
}

impl<Key> PartialEq<Key> for BranchCell<Key> 
where Key: PartialEq
{
    fn eq(&self, other: &Key) -> bool {
        self.key == *other
    }
}

impl<Key> PartialOrd<Key> for BranchCell<Key> 
where Key: PartialOrd + PartialEq
{
    fn partial_cmp(&self, other: &Key) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other)
    }
}

pub struct Branch<Key>{
    id: BPTreeNodeId,
    capacity: usize,
    cells: Vec<BranchCell<Key>> 
}

impl<K> Branch<K> 
where K: Default + Copy + PartialOrd
{
    pub fn new(id: BPTreeNodeId, capacity: usize, split: Split<K>) -> Self {
        Self { 
            id: id, 
            capacity: capacity, 
            cells: vec![
                BranchCell::from((split.0, split.1)),
                BranchCell::from((split.2, Default::default()))
            ] 
        }
    }

    pub fn new_with_cells<Iter>(id: BPTreeNodeId, capacity: usize, cells: Iter) -> Self 
    where Iter: Iterator<Item=(BPTreeNodeId, K)>
    {
        Self {
            id: id,
            capacity: capacity,
            cells: cells.map(BranchCell::from).collect()
        }
    }
    
    /// Check if the node is overflowing.
    pub fn is_overflowing(&self) -> bool {
        self.cells.len() >= self.capacity
    }

    pub fn search_cell(&self, left: BPTreeNodeId) -> Option<usize> {
        self.cells
        .iter()
        .enumerate()
        .find(|(_, c)| c.left == left)
        .map(|(i, _)| i)        
    }

    pub fn insert(&mut self, split: Split<K>) {
        let cidx = self.search_cell(split.0).unwrap();
        let key = self.cells[cidx].key;
        self.cells.insert(cidx, BranchCell { left: split.2, key });
        self.cells[cidx + 1].left = split.0;
        self.cells[cidx + 1].key = split.1;
    }

    pub fn search(&self, key: &K) -> Option<BPTreeNodeId> {
        self.cells
        .iter()
        .find(|c| *c < key)
        .map(|c| c.left)
    }

    /// Split the node
    pub fn split<Nodes>(&mut self, nodes: &mut Nodes) -> Split<Nodes::Key>
    where Nodes: BPTreeNodes<Key=K> {
        let middle: usize = self.cells.len() / 2;
        let right_cells = self.cells.drain(middle+1..self.cells.len()).map(BranchCell::into);
        let right = nodes.new_branch_with_cells(self.capacity, right_cells);

        let middle_cell = self.cells.last_mut().unwrap();
        let key = middle_cell.key;
        middle_cell.key = Default::default(); 

        (self.id, key, right)
    }
}