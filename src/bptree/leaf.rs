use super::{node::{BPTreeNodeId}, nodes::traits::{BPTreeNodes, Split}, result::BPTreeResult, error::BPTreeError};

pub struct LeafCell<Key, Value>
{
    key: Key,
    value: Value
}

impl<K, E> From<(K,E)> for LeafCell<K, E> {
    fn from(value: (K,E)) -> Self {
        Self { key: value.0, value: value.1 }
    }
}

impl<Key, Value> PartialEq<Key> for LeafCell<Key, Value> 
where Key: PartialEq
{
    fn eq(&self, other: &Key) -> bool {
        self.key == *other
    }
}

impl<Key, Value> PartialOrd<Key> for LeafCell<Key, Value> 
where Key: PartialOrd
{
    fn partial_cmp(&self, other: &Key) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other)
    }
}

pub struct Leaf<Key, Value> {
    id: BPTreeNodeId,
    capacity: usize,
    cells: Vec<LeafCell<Key, Value>>,
    next: Option<BPTreeNodeId>
}

impl<K, V> Leaf<K,V> 
where K: Copy + PartialEq + PartialOrd
{
    pub fn new(id: BPTreeNodeId, capacity: usize, key: K, element: V) -> Self {
        Self { 
            id: id, 
            capacity: capacity, 
            cells: vec![LeafCell::from((key, element))],
            next: None
        }
    }

    pub fn new_with_cells<Iter>(id: BPTreeNodeId, capacity: usize, cells: Iter) -> Self 
    where Iter: Iterator<Item=(K, V)> {
        Self {
            id: id,
            capacity: capacity,
            cells: cells.map(LeafCell::from).collect(),
            next: None
        }
    }
    
    /// Check if the node is overflowing.
    pub fn is_overflowing(&self) -> bool {
        self.cells.len() >= self.capacity
    }

    /// Search the cell which is the maximum of the cells which key is lower than the given key.
    pub fn search_nearest_cell(&self, key: &K) -> Option<usize> {
        self.cells
        .iter()
        .enumerate()
        .find(|(_, c)| *c >= key)
        .map(|(i, _)| i)      
    }

    /// Search the cell containing the key
    pub fn search_cell(&self, key: &K) -> Option<usize> {
        self.cells
        .iter()
        .enumerate()
        .find(|(_, c)| *c == key)
        .map(|(i, _)| i)      
    }

    /// Update or insert (key, value) tuple.
    pub fn upsert(&mut self, key: K, value: V) {
        match self.search_nearest_cell(&key) {
            Some(cell_index) => {
                if self.cells[cell_index].key == key {
                    self.cells[cell_index].value = value;
                } else {
                    self.cells.insert(cell_index, LeafCell { key: key, value: value });
                }
            }
            None => {
                self.cells.push(LeafCell { key: key, value: value });
            }
        }
    }

    /// Update (key, value) tuple.
    pub fn update(&mut self, key: &K, value: V) -> BPTreeResult<()> {
        match self.search_nearest_cell(&key) {
            Some(cell_index) => {
                if self.cells[cell_index].key == *key {
                    self.cells[cell_index].value = value;
                    return Ok(())
                } else {
                    return Err(BPTreeError::KeyNotFound);
                }
            }
            None => return Err(BPTreeError::KeyNotFound)
        }
    }

    /// Update or insert (key, value) tuple.
    /// Return an error if the key already exists.
    pub fn insert(&mut self, key: K, value: V) -> BPTreeResult<()> {
        match self.search_nearest_cell(&key) {
            Some(cell_index) => {
                if self.cells[cell_index].key == key {
                    return Err(super::error::BPTreeError::ExistingKey)
                } else {
                    self.cells.insert(cell_index, LeafCell { key: key, value: value });
                    return Ok(())
                }
            }
            None => {
                self.cells.push(LeafCell { key: key, value: value });
                return Ok(())
            }
        }
    }

    /// Check if the leaf contains the key
    pub fn contains(&self, key: &K) -> bool {
        self.search_cell(key).is_some()
    }

    /// Split the leaf into two leaves.
    pub fn split<Nodes>(&mut self, nodes: &mut Nodes) -> Split<Nodes::Key>
    where Nodes: BPTreeNodes<Key=K, Value=V>
    {
        let middle: usize = self.cells.len() / 2;
        let right_cells: Vec<LeafCell<K, V>> = self.cells.drain(middle..self.cells.len()).collect();
        let middle_key = right_cells.first().unwrap().key;

        let right_leaf = nodes.new_leaf_with_cells(
            self.capacity, 
            right_cells.into_iter().map(|c| (c.key, c.value))
        );
        self.next = Some(right_leaf);
        (self.id, middle_key, right_leaf)
    }
}
