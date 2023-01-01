use crate::object::ObjectId;

pub type BPTreeNodeId = ObjectId;



pub mod traits {
    use super::BPTreeNodeId;

    pub type LeafCell<K,E> = (K,E);
    pub type BranchCell<K> = (BPTreeNodeId, K);

    pub trait BPTreeNodes {
        type Key;
        type Element;
        
        /// Create a branch with prexisting cells.
        fn new_branch_with_cells<Iter>(&mut self, capacity: usize, cells: Iter) -> BPTreeNodeId
        where Iter: Iterator<Item=BranchCell<Self::Key>>;

        /// Create a leaf with prexisting cells.
        fn new_leaf_with_cells<Iter>(&mut self, capacity: usize, cells: Iter) -> BPTreeNodeId
        where Iter: Iterator<Item=LeafCell<Self::Key, Self::Element>>;

        /// Insert a cell in a branch node.
        fn branch_insert<Iter>(&mut self, branch: BPTreeNodeId, left: BPTreeNodeId, key: Self::Key, right: BPTreeNodeId);
        
        /// Is the node a leaf ?
        fn is_leaf(&self, id: BPTreeNodeId) -> bool;

        /// Is the node a branch ?
        fn is_branch(&self, id: BPTreeNodeId) -> bool;
    }
}