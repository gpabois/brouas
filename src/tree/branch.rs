use std::marker::PhantomData;

use self::traits::Branch as TraitBranch;
use crate::tree::node::traits::BorrowNode;
use super::{cells::branch::BranchCells, NodeRef};
use super::cells::branch::traits::BranchCells as TraitBranchCells;

pub mod traits {
    use crate::tree::node::traits::Node;
    use crate::tree::node_ref::NodeRef;
    use crate::tree::node::traits::BorrowNode;

    /// A branch of a Merkle B+ Tree
    pub trait Branch
    {
        type Node: Node;

        /// Create a branch
        fn new(left: NodeRef<<Self::Node as Node>::Hash>, key: <Self::Node as Node>::Key, right: NodeRef<<Self::Node as Node>::Hash>) -> Self;

        /// Insert a cell into the branch
        fn insert(&mut self, left: NodeRef<<Self::Node as Node>::Hash>, key: <Self::Node as Node>::Key, right: NodeRef<<Self::Node as Node>::Hash>);
        
        /// Search the node satifying the key
        fn search_node<'a>(&'a self, key: &<Self::Node as Node>::Key) -> &'a NodeRef<<Self::Node as Node>::Hash>;

        /// Split the branch, and returns right node
        fn split_branch(&mut self) -> (<Self::Node as Node>::Key, Self) where Self: Sized;

        /// Returns the children refs
        fn children<'a>(&'a self) -> Vec<&'a NodeRef<<Self::Node as Node>::Hash>>;

        /// Compute the hash
        fn compute_hash<Nodes: BorrowNode<Self::Node>>(&self, nodes: &Nodes) -> <Self::Node as Node>::Hash;

        /// 
        fn is_full(&self) -> bool;
    }
     
}

pub struct Branch<Node>
where Node: crate::tree::node::traits::Node
{
    cells: BranchCells<Node>
}

impl<Node> Branch<Node>
where Node: crate::tree::node::traits::Node
{
    fn new_from_cells(cells: BranchCells<Node>) -> Self {
        Self {cells: cells}
    } 
}

impl<Node> TraitBranch for Branch<Node>
where Node: crate::tree::node::traits::Node
{
    type Node = Node;

    fn new(left: super::NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>, key: <Self::Node as crate::tree::node::traits::Node>::Key, right: super::NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>) -> Self 
    {
        Self {
            cells: BranchCells::new(left, key, right)
        }
    }

    fn insert(&mut self, left: super::NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>, key: <Self::Node as crate::tree::node::traits::Node>::Key, right: super::NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>) 
    {
        self.cells.insert(left, key, right)
    }

    /// Search the node satifying the key
    fn search_node<'a>(&'a self, key: &<Self::Node as crate::tree::node::traits::Node>::Key) -> &'a NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>
    {
        self.cells.search(key)
    }

    /// Split the branch, and returns right node
    fn split_branch(&mut self) -> (<Self::Node as crate::tree::node::traits::Node>::Key, Self) where Self: Sized
    {
        let (key, right_cells) = self.cells.split();
        (key, Self::new_from_cells(right_cells))
    }

    fn children<'a>(&'a self) -> Vec<&'a super::NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>> 
    {
        todo!()
    }

    fn compute_hash<Nodes: BorrowNode<Self::Node>>(&self, _nodes: &Nodes) -> <Self::Node as crate::tree::node::traits::Node>::Hash 
    {
        todo!()
    }

    fn is_full(&self) -> bool {
        self.cells.is_full()
    }
}