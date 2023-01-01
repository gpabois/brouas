use super::node::{BPTreeNodeId, traits::{BPTreeNodes}};

pub struct LeafCell<Key, Element>
{
    key: Key,
    element: Element
}

impl<K, E> From<(K,E)> for LeafCell<K, E> {
    fn from(value: (K,E)) -> Self {
        Self { key: value.0, element: value.1 }
    }
}

impl<Key, Element> PartialEq for LeafCell<Key, Element> 
where Key: PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key 
    }
}

impl<Key, Element> PartialOrd for LeafCell<Key, Element> 
where Key: PartialOrd
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

pub struct Leaf<Key, Element> {
    id: BPTreeNodeId,
    capacity: usize,
    cells: Vec<LeafCell<Key, Element>>
}

impl<K, E> Leaf<K,E> 
where K: Copy
{
    pub fn new(id: BPTreeNodeId, capacity: usize, key: K, element: E) -> Self {
        Self { 
            id: id, 
            capacity: capacity, 
            cells: vec![LeafCell::from((key, element))] 
        }
    }

    pub fn new_with_cells<Iter>(id: BPTreeNodeId, capacity: usize, cells: Iter) -> Self 
    where Iter: Iterator<Item=(K, E)> {
        Self {
            id: id,
            capacity: capacity,
            cells: cells.map(LeafCell::from).collect()
        }
    }
    
    pub fn is_overflowing(&self) -> bool {
        self.cells.len() >= self.capacity
    }

    /// Split the leaf into two leaves.
    pub fn split<Nodes>(&mut self, nodes: &mut Nodes) -> (BPTreeNodeId, K, BPTreeNodeId)
    where Nodes: BPTreeNodes<Key=K, Element=E>
    {
        let middle: usize = self.cells.len() / 2;
        let right_cells: Vec<LeafCell<K, E>> = self.cells.drain(middle..self.cells.len()).collect();
        let middle_key = right_cells.first().unwrap().key;

        let right_leaf = nodes.new_leaf_with_cells(
            self.capacity, 
            right_cells.into_iter().map(|c| (c.key, c.element))
        );

        (self.id, middle_key, right_leaf)
    }
}
