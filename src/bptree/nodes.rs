use std::ops::Deref;

use crate::utils::{Watcher, collections::FrozenBTreeMap};

use super::{node::BPTreeNodeId, leaf::Leaf, branch::Branch};


pub mod traits {
    use crate::bptree::node::BPTreeNodeId;

    pub type LeafCell<K,V> = (K,V);
    pub type BranchCell<K> = (BPTreeNodeId, K);

    pub type Split<K> = (BPTreeNodeId, K, BPTreeNodeId);

    pub trait BPTreeNodes {
        type Key;
        type Value;
        
        /// Create a branch with prexisting cells.
        fn new_branch_with_cells<Iter>(&mut self, capacity: usize, cells: Iter) -> BPTreeNodeId
        where Iter: Iterator<Item=BranchCell<Self::Key>>;

        /// Create a branch.
        fn new_branch(&mut self, capacity: usize, split: Split<Self::Key>) -> BPTreeNodeId;

        /// Create a leaf with prexisting cells.
        fn new_leaf_with_cells<Iter>(&mut self, capacity: usize, cells: Iter) -> BPTreeNodeId
        where Iter: Iterator<Item=LeafCell<Self::Key, Self::Value>>;

        /// Create a new leaf.
        fn new_leaf(&mut self, capacity: usize, key: Self::Key, element: Self::Value) -> BPTreeNodeId;

        /// Insert a cell in a branch node.
        fn branch_insert(&mut self, branch: BPTreeNodeId, split: Split<Self::Key>);
        
        /// Get the child
        fn branch_search(&self, branch: BPTreeNodeId, key: &Self::Key) -> Option<BPTreeNodeId>;

        /// Insert a cell in a leaf node.
        fn leaf_insert(&mut self, leaf: BPTreeNodeId, key: Self::Key, value: Self::Value);

        /// Update a cell in a leaf node.
        fn leaf_update(&mut self, leaf: BPTreeNodeId, key: &Self::Key, value: Self::Value);

        /// Split a node
        fn split(&mut self, node: BPTreeNodeId) -> Split<Self::Key>;

        /// Is the node a leaf ?
        fn is_leaf(&self, id: BPTreeNodeId) -> bool;

        /// Is the node a branch ?
        fn is_branch(&self, id: BPTreeNodeId) -> bool;

        /// Is the node overflowing ?
        fn is_overflowing(&self, id: BPTreeNodeId) -> bool;
    }
}

pub struct BPTreeNodes<K,V> {
    counter: BPTreeNodeId,
    leaves: FrozenBTreeMap<BPTreeNodeId, Watcher<Leaf<K,V>>>,
    branches: FrozenBTreeMap<BPTreeNodeId, Watcher<Branch<K>>>
}

impl<K,V> BPTreeNodes<K,V> 
{
    pub fn new() -> Self {
        Self {
            counter: Default::default(),
            leaves: Default::default(),
            branches: Default::default()
        }
    }

    pub fn set_node_counter(&mut self, counter: BPTreeNodeId) {
        self.counter = counter;
    }

    pub fn new_node_id(&mut self) -> BPTreeNodeId {
        self.counter += 1;
        self.counter
    }

    pub fn contains_branch(&self, id: &BPTreeNodeId) -> bool {
        self.branches.contains_key(id)
    }
    
    pub fn contains_leaf(&self, id: &BPTreeNodeId) -> bool {
        self.leaves.contains_key(id)
    }
}

impl<K,V> self::traits::BPTreeNodes for BPTreeNodes<K,V> 
where K: Default + Copy + PartialEq + PartialOrd
{
    type Key = K;
    type Value = V;

    fn new_branch_with_cells<Iter>(&mut self, capacity: usize, cells: Iter) -> BPTreeNodeId
        where Iter: Iterator<Item=traits::BranchCell<Self::Key>> {
        let nid = self.new_node_id();
        let branch = Branch::new_with_cells(nid, capacity, cells);
        self.branches.insert(nid, Watcher::new(branch));
        nid
    }

    fn new_branch(&mut self, capacity: usize, split: traits::Split<Self::Key>) -> BPTreeNodeId {
        let nid = self.new_node_id();
        let branch = Branch::new(nid, capacity, split);
        self.branches.insert(nid, Watcher::new(branch));
        nid
    }

    fn new_leaf_with_cells<Iter>(&mut self, capacity: usize, cells: Iter) -> BPTreeNodeId
        where Iter: Iterator<Item=traits::LeafCell<Self::Key, Self::Value>> {
        let nid = self.new_node_id();
        let leaf = Leaf::new_with_cells(nid, capacity, cells);
        self.leaves.insert(nid, Watcher::new(leaf));
        nid
    }

    fn new_leaf(&mut self, capacity: usize, key: Self::Key, value: Self::Value) -> BPTreeNodeId {
        let nid = self.new_node_id();
        let leaf = Leaf::new(nid, capacity, key, value);
        self.leaves.insert(nid, Watcher::new(leaf));
        nid
    }

    fn branch_insert(&mut self, branch: BPTreeNodeId, split: traits::Split<Self::Key>) {
        self.branches.get_mut(&branch).unwrap().insert(split)
    }

    fn branch_search(&self, branch: BPTreeNodeId, key: &Self::Key) -> Option<BPTreeNodeId> {
        self.branches.get(&branch).unwrap().search(key)
    }

    fn leaf_insert(&mut self, leaf: BPTreeNodeId, key: Self::Key, value: Self::Value) {
        self.leaves.get_mut(&leaf).unwrap().insert(key, value);
    }

    fn leaf_update(&mut self, leaf: BPTreeNodeId, key: &Self::Key, value: Self::Value) {
        self.leaves.get_mut(&leaf).unwrap().update(key, value);
    }

    fn split(&mut self, node: BPTreeNodeId) -> traits::Split<Self::Key> {
        if self.is_leaf(node) {
            self.leaves.get_mut(&node).unwrap().split(self)
        } else {
            self.branches.get_mut(&node).unwrap().split(self)
        }
    }

    fn is_leaf(&self, id: BPTreeNodeId) -> bool {
        self.leaves.contains_key(&id)
    }

    fn is_branch(&self, id: BPTreeNodeId) -> bool {
        self.branches.contains_key(&id)
    }

    fn is_overflowing(&self, id: BPTreeNodeId) -> bool {
        if self.contains_branch(&id) {
            self.branches.get(&id).unwrap().is_overflowing()
        } else {
            self.leaves.get(&id).unwrap().is_overflowing()
        }
    }
}