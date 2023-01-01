use super::node::{BPTreeNodeId, traits::BPTreeNodes};

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

impl<Key> PartialEq for BranchCell<Key> 
where Key: PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key 
    }
}

impl<Key> PartialOrd for BranchCell<Key> 
where Key: PartialOrd
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

pub struct Branch<Key>{
    id: BPTreeNodeId,
    capacity: usize,
    cells: Vec<BranchCell<Key>> 
}

impl<K> Branch<K> 
where K: Default + Copy
{
    pub fn new(id: BPTreeNodeId, capacity: usize, left: BPTreeNodeId, key: K, right: BPTreeNodeId) -> Self {
        Self { 
            id: id, 
            capacity: capacity, 
            cells: vec![
                BranchCell::from((left, key)),
                BranchCell::from((right, Default::default()))
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

    /// Split the node
    pub fn split<Nodes>(&mut self, nodes: &mut Nodes) -> (BPTreeNodeId, K, BPTreeNodeId)
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