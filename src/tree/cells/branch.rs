use self::traits::BranchCells as TraitBranchCells;
use crate::tree::node::traits::Node as TNode;
use crate::tree::nodes::traits::Nodes as TNodes;

use crate::tree::result::TreeResult;
use crate::{hash::traits::{Hash, Hashable, Hasher}, tree::node_ref::WeakNode};

pub mod traits {
    use crate::tree::node_ref::WeakNode;
    use crate::tree::node::traits::Node as TNode;
    use crate::tree::nodes::traits::Nodes as TNodes;
    use crate::tree::result::TreeResult;

    pub trait BranchCells
    {
        type Node: TNode;
        
        /// New branch cells
        fn new(left: WeakNode< Self::Node>, key: <Self::Node as TNode>::Key, right: WeakNode< Self::Node>) -> Self;
        /// Search the node based on the key
        fn search(&self, k: &<Self::Node as TNode>::Key) -> &WeakNode< Self::Node>;
        /// Split the cells
        fn split(&mut self) -> (Self, <Self::Node as TNode>::Key, Self) where Self: Sized;
        /// The cells are full
        fn is_full(&self) -> bool;
        /// Insert a cell
        fn insert(&mut self, place: &WeakNode< Self::Node>, left: WeakNode< Self::Node>, key: <Self::Node as TNode>::Key, right: WeakNode< Self::Node>);
        /// Compute the branch cells hash
        fn compute_hash<Nodes: TNodes< Node=Self::Node>>(&self, nodes: &Nodes) -> TreeResult<<Self::Node as TNode>::Hash, Self::Node>;
        /// Return the weak nodes references
        fn nodes(&self) -> Vec<&WeakNode< Self::Node>>;
        /// Return the mutable weak nodes references
        fn mut_nodes(&mut self) -> Vec<&mut WeakNode< Self::Node>>;
    }
}

pub struct BranchCells< Node>
where Node: TNode
{
    head: WeakNode< Node>,
    cells: Vec<BranchCell< Node>>
} 

impl< Node> TraitBranchCells for BranchCells< Node>
where Node: TNode
{
    type Node = Node;

    fn new(left: WeakNode< Self::Node>, key: <Self::Node as TNode>::Key, right: WeakNode<Self::Node>) -> Self {
        Self {
            head: left,
            cells: vec![BranchCell(key, right)]
        }
    }

    fn search(&self, k: &<Self::Node as TNode>::Key) -> &WeakNode< Self::Node> {
        let mut node = &self.head;
        if let Some(n) = self.cells
        .iter()
        .filter(|c| {c <= &k})
        .last().map(|c| &c.1) 
        {
            node = n
        }
        node
    }

    fn split(&mut self) -> (Self, <Self::Node as TNode>::Key, Self) {
        let middle_index = <Self::Node as crate::tree::node::traits::Node>::SIZE/2;

        let lefts: Vec<_> = self.cells.drain(0..middle_index).collect();
        let rights: Vec<_> = self.cells.drain(1..).collect();

        let middle_cell = self.cells.pop().unwrap();
        let middle_key = middle_cell.0;
        
        let right_cell = Self {
            head: middle_cell.1,
            cells: rights.into_iter().collect()
        };

        let left_cell = Self {
            head: self.head.to_owned(),
            cells: lefts
        };

        return (left_cell, middle_key, right_cell)
    }

    fn is_full(&self) -> bool {
        self.cells.len() >= <Self::Node as crate::tree::node::traits::Node>::SIZE
    }

    fn insert(&mut self, place: &WeakNode< Self::Node>, left: WeakNode< Self::Node>, key: <Self::Node as TNode>::Key, right: WeakNode< Self::Node>) 
    {
        let (idx, cell) = self.cells
        .iter_mut()
        .enumerate()
        .find(|(_, cell)| std::cmp::PartialEq::eq(place, &(*cell).1))
        .unwrap();
        cell.1 = left;
        self.cells.insert(idx + 1, BranchCell(key, right));
    }

    fn compute_hash<Nodes: TNodes< Node=Self::Node>>(&self, nodes: &Nodes) -> TreeResult<<Self::Node as TNode>::Hash, Self::Node>
    {
        let mut hasher = <Self::Node as TNode>::Hash::new_hasher();

        self.head.get_hash(nodes)?.unwrap().hash(&mut hasher);
 
        self.cells.iter().for_each(|cell| {
            cell.0.hash(&mut hasher);
            cell.1.get_hash(nodes).unwrap().unwrap().hash(&mut hasher)
        });

        Ok(hasher.finalize())
    }

    fn nodes(&self) -> Vec<&WeakNode<Self::Node>> {
        todo!()
    }

    fn mut_nodes(&mut self) -> Vec<&mut WeakNode< Self::Node>> {
        todo!()
    }
}

#[derive(Default)]
pub struct BranchCell< Node>(Node::Key, WeakNode< Node>)
where Node: TNode;

impl< Node: TNode> std::cmp::PartialOrd<Node::Key> for BranchCell< Node>
{
    fn partial_cmp(&self, other: &Node::Key) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl< Node: TNode>  std::cmp::PartialOrd<&Node::Key> for &mut BranchCell< Node>
{
    fn partial_cmp(&self, other: &&Node::Key) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl< Node: TNode>  std::cmp::PartialEq<Node::Key> for BranchCell< Node>
{
    fn eq(&self, other: &Node::Key) -> bool {
        self.0 == *other
    }
}
