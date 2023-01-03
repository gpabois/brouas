use crate::{io::{traits::{OutStream, InStream}, DataStream}, object::{BPTREE_LEAF}};

use super::{node::{BPTreeNodeId}, nodes::traits::{BPTreeNodes, Split}, result::BPTreeResult, error::BPTreeError};

pub struct LeafCell<Key, Value>
{
    key: Key,
    value: Value
}

impl<K,V> Clone for LeafCell<K,V> where K: Clone, V: Clone {
    fn clone(&self) -> Self {
        Self { key: self.key.clone(), value: self.value.clone() }
    }
}

impl<K,V> Default for LeafCell<K,V> where K: Default, V: Default {
    fn default() -> Self {
        Self { key: Default::default(), value: Default::default() }
    }
}

impl<K, V> OutStream for LeafCell<K, V>
where K: OutStream<Output=K>, V: OutStream<Output=V> {
    type Output = Self;

    fn write_to_stream<W: std::io::Write + ?Sized>(output: &Self, writer: &mut W) -> std::io::Result<usize> {
        Ok(
            K::write_to_stream(&output.key, writer)? +
            V::write_to_stream(&output.value, writer)?
        )
    }

    fn write_all_to_stream<W: std::io::Write + ?Sized>(output: &Self, writer: &mut W) -> std::io::Result<()> {
        K::write_all_to_stream(&output.key, writer)?;
        V::write_all_to_stream(&output.value, writer)
    }
}

impl <K, V> InStream for LeafCell<K, V> 
where K: InStream<Input=K>, V: InStream<Input=V>
{
    type Input = Self;

    fn read_from_stream<R: std::io::Read + ?Sized>(input: &mut Self, read: &mut R) -> std::io::Result<()> {
        K::read_from_stream(&mut input.key, read)?;
        V::read_from_stream(&mut input.value, read)
    }
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

impl<K,V> InStream for Leaf<K,V> 
where K: InStream<Input=K> + Default + Clone, V: InStream<Input=V> + Default + Clone
{
    type Input = Self;

    fn read_from_stream<R: std::io::Read + ?Sized>(input: &mut Self, read: &mut R) -> std::io::Result<()> 
    {
        input.capacity = DataStream::<u64>::read(read)? as usize;
        input.cells = vec![LeafCell::default(); DataStream::<u64>::read(read)? as usize];
        input.next = DataStream::<u64>::read(read).map(|u| match u {
            0 => None,
            id => Some(id)
        })?;

        for c in input.cells.iter_mut() {
            LeafCell::<K,V>::read_from_stream(c, read)?;
        }

        Ok(())
    }
}

impl<K,V> OutStream for Leaf<K, V>
where K: OutStream<Output=K>, V: OutStream<Output=V>
{
    type Output = Self;

    fn write_to_stream<W: std::io::Write + ?Sized>(output: &Self, writer: &mut W) -> std::io::Result<usize> {
        let mut written = DataStream::<u64>::write(writer, output.capacity as u64)? +
            DataStream::<u64>::write(writer, output.cells.len() as u64)? +
            DataStream::<u64>::write(writer, output.next.unwrap_or(0))?;

        for c in output.cells.iter() {
            written += LeafCell::<K,V>::write_to_stream(c, writer)?;
        }

        Ok(written)
    }

    fn write_all_to_stream<W: std::io::Write + ?Sized>(input: &Self, writer: &mut W) -> std::io::Result<()> {
        DataStream::<u64>::write_all(writer, input.capacity as u64)?;
        DataStream::<u64>::write_all(writer, input.cells.len() as u64)?;
        DataStream::<u64>::write_all(writer, input.next.unwrap_or(0))?;

        for c in input.cells.iter() {
            LeafCell::<K,V>::write_all_to_stream(c, writer)?;
        }

        Ok(())
    }
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
    pub fn split<Nodes>(&mut self, nodes: &Nodes) -> Split<Nodes::Key>
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
