use std::{ops::{Deref, DerefMut}, cell::RefCell};
use crate::utils::{Watcher, collections::FrozenBTreeMap};

use super::{node::BPTreeNodeId, leaf::Leaf, branch::Branch, result::BPTreeResult, error::BPTreeError};


pub mod traits {
    use crate::bptree::{node::BPTreeNodeId, result::BPTreeResult};

    pub type LeafCell<K,V> = (K,V);
    pub type BranchCell<K> = (BPTreeNodeId, K);

    pub type Split<K> = (BPTreeNodeId, K, BPTreeNodeId);

    pub trait BPTreeNodes {
        type Key;
        type Value;
        
        /// Create a branch with prexisting cells.
        fn new_branch_with_cells<Iter>(&self, capacity: usize, cells: Iter) -> BPTreeNodeId
        where Iter: Iterator<Item=BranchCell<Self::Key>>;

        /// Create a branch.
        fn new_branch(&self, capacity: usize, split: Split<Self::Key>) -> BPTreeNodeId;

        /// Create a leaf with prexisting cells.
        fn new_leaf_with_cells<Iter>(&self, capacity: usize, cells: Iter) -> BPTreeNodeId
        where Iter: Iterator<Item=LeafCell<Self::Key, Self::Value>>;

        /// Create a new leaf.
        fn new_leaf(&self, capacity: usize, key: Self::Key, element: Self::Value) -> BPTreeNodeId;

        /// Insert a cell in a branch node.
        fn branch_insert(&mut self, branch: BPTreeNodeId, split: Split<Self::Key>);
        
        /// Get the child
        fn branch_search(&self, branch: BPTreeNodeId, key: &Self::Key) -> Option<BPTreeNodeId>;

        /// Insert a cell in a leaf node.
        fn leaf_insert(&mut self, leaf: BPTreeNodeId, key: Self::Key, value: Self::Value) -> BPTreeResult<()>;

        /// Update a cell in a leaf node.
        fn leaf_update(&mut self, leaf: BPTreeNodeId, key: &Self::Key, value: Self::Value) -> BPTreeResult<()>;

        /// The leaf contains the keys.
        fn leaf_contains(&self, leaf: BPTreeNodeId, key: &Self::Key) -> BPTreeResult<bool>;

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
    counter: RefCell<BPTreeNodeId>,
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
        *self.counter.borrow_mut() = counter;
    }

    pub fn new_node_id(&self) -> BPTreeNodeId {
        *self.counter.borrow_mut() += 1;
        *self.counter.borrow()
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

    fn new_branch_with_cells<Iter>(&self, capacity: usize, cells: Iter) -> BPTreeNodeId
        where Iter: Iterator<Item=traits::BranchCell<Self::Key>> {
        let nid = self.new_node_id();
        let branch = Branch::new_with_cells(nid, capacity, cells);
        self.branches.insert(nid, Watcher::new(branch));
        nid
    }

    fn new_branch(&self, capacity: usize, split: traits::Split<Self::Key>) -> BPTreeNodeId {
        let nid = self.new_node_id();
        let branch = Branch::new(nid, capacity, split);
        self.branches.insert(nid, Watcher::new(branch));
        nid
    }

    fn new_leaf_with_cells<Iter>(&self, capacity: usize, cells: Iter) -> BPTreeNodeId
        where Iter: Iterator<Item=traits::LeafCell<Self::Key, Self::Value>> {
        let nid = self.new_node_id();
        let leaf = Leaf::new_with_cells(nid, capacity, cells);
        self.leaves.insert(nid, Watcher::new(leaf));
        nid
    }

    fn new_leaf(&self, capacity: usize, key: Self::Key, value: Self::Value) -> BPTreeNodeId {
        let nid = self.new_node_id();
        let leaf = Leaf::new(nid, capacity, key, value);
        self.leaves.insert(nid, Watcher::new(leaf));
        nid
    }

    fn branch_insert(&mut self, branch: BPTreeNodeId, split: traits::Split<Self::Key>) {
        unsafe {
            self.branches
            .get(&branch)
            .unwrap()
            .deref()
            .borrow_mut()
            .deref_mut()
            .as_mut()
            .get_unchecked_mut()
            .insert(split)

        }
    }

    fn branch_search(&self, branch: BPTreeNodeId, key: &Self::Key) -> Option<BPTreeNodeId> {
        self.branches.get(&branch).unwrap().deref().borrow().search(key)
    }

    fn leaf_insert(&mut self, leaf: BPTreeNodeId, key: Self::Key, value: Self::Value) -> BPTreeResult<()> {
        unsafe {
            self.leaves.get(&leaf)
            .ok_or(BPTreeError::LeafNotFound)?
            .deref()
            .borrow_mut()
            .deref_mut()
            .as_mut()
            .get_unchecked_mut()
            .insert(key, value)
        }

    }

    fn leaf_update(&mut self, leaf: BPTreeNodeId, key: &Self::Key, value: Self::Value) -> BPTreeResult<()> {
        unsafe {
            self.leaves.get(&leaf)
            .ok_or(BPTreeError::LeafNotFound)?
            .deref()
            .borrow_mut()
            .as_mut()
            .get_unchecked_mut()
            .update(key, value)
        }
    }

    fn leaf_contains(&self, leaf: BPTreeNodeId, key: &Self::Key) -> BPTreeResult<bool> {
        Ok(
            self.leaves
            .get(&leaf)
            .ok_or(BPTreeError::LeafNotFound)?
            .deref()
            .borrow()
            .contains(key)
        )
    }

    fn split(&mut self, node: BPTreeNodeId) -> traits::Split<Self::Key> {
        unsafe {
            if self.is_leaf(node) {
                self.leaves
                .get(&node)
                .unwrap()
                .deref()
                .borrow_mut()
                .as_mut()
                .get_unchecked_mut()
                .split(self)
            } else {
                self.branches
                .get(&node)
                .unwrap()
                .deref()
                .borrow_mut()
                .as_mut()
                .get_unchecked_mut()
                .split(self)
            }
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
            self.branches.get(&id).unwrap().deref().borrow().is_overflowing()
        } else {
            self.leaves.get(&id).unwrap().deref().borrow().is_overflowing()
        }
    }
}