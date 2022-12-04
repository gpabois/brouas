use self::traits::BranchCells as TraitBranchCells;
use crate::tree::node::traits::Node as TNode;
use crate::{hash::traits::{Hash, Hasher, Hashable}, tree::node_ref::NodeRef};

pub mod traits {
    use crate::tree::node_ref::NodeRef;
    use crate::tree::node::traits::Node as TNode;

    pub trait BranchCells<'a>
    {
        type Node: crate::tree::node::traits::Node;
        
        /// New branch cells
        fn new(left: NodeRef<'a, Self::Node>, key: <Self::Node as TNode>::Key, right: NodeRef<'a, Self::Node>) -> Self;
        /// Search the node based on the key
        fn search<'b>(&'b self, k: &<Self::Node as TNode>::Key) -> &'b NodeRef<'b, Self::Node>;
        /// Split the cells
        fn split(&mut self) -> (Self, <Self::Node as TNode>::Key, Self) where Self: Sized;
        /// The cells are full
        fn is_full(&self) -> bool;
        /// Insert a cell
        fn insert(&mut self, left: NodeRef<'a, Self::Node>, key: <Self::Node as TNode>::Key, right: NodeRef<'a, Self::Node>);
        /// Compute the branch cells hash
        fn compute_hash(&self) -> <Self::Node as TNode>::Hash;
    }
}

pub struct BranchCells<'a, Node>
where Node: crate::tree::node::traits::Node
{
    head: NodeRef<'a, Node>,
    cells: Vec<BranchCell<'a, Node>>
} 

impl<'a, Node> TraitBranchCells<'a> for BranchCells<'a, Node>
where Node: crate::tree::node::traits::Node
{
    type Node = Node;
    fn search<'b>(&'b self, k: &<Self::Node as crate::tree::node::traits::Node>::Key) -> &'b NodeRef<'a, Self::Node>
    {
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
 

    fn split(&mut self) -> (Self, <Self::Node as crate::tree::node::traits::Node>::Key, Self) {
        let middle_index = <Self::Node as crate::tree::node::traits::Node>::SIZE/2;

        let slice = self.cells.into_boxed_slice();
        
        let mut lefts: Vec<BranchCell<Self::Node>> = Vec::with_capacity(middle_index - 1);
        lefts.copy_from_slice(&slice[0..middle_index]);

        let mut rights: Vec<BranchCell<Self::Node>> = Vec::with_capacity(middle_index - 1);
        rights.as_ref().copy_from_slice(&slice[middle_index + 1 ..]);

        let middle_cell = slice[middle_index];

        let middle_key = middle_cell.0;
        let right_cell = Self {
            head: middle_cell.1,
            cells: rights.into_iter().collect()
        };

        let left_cell = Self {
            head: self.head,
            cells: lefts
        };

        return (left_cell, middle_key, right_cell)
    }

    fn is_full(&self) -> bool {
        self.cells.len() >= <Self::Node as crate::tree::node::traits::Node>::SIZE
    }

    fn insert(&mut self, left: NodeRef<'a, Self::Node>, key: <Self::Node as TNode>::Key, right: NodeRef<'a, Self::Node>) {
        let (idx, cell) = self.cells
        .iter_mut()
        .enumerate()
        .find(|(_idx, cell)| cell.1 == left)
        .expect("Expecting to find node ref in tree branch");

        let right_key = cell.0.clone();
        cell.0 = key;

        self.cells.insert(idx + 1, BranchCell(right_key, right));
    }

    fn new(left: NodeRef<'a, Self::Node>, key: <Self::Node as TNode>::Key, right: NodeRef<'a, Self::Node>) -> Self {
        Self {
            head: left,
            cells: vec![BranchCell(key, right)]
        }
    }

    fn compute_hash(&self) -> <Self::Node as TNode>::Hash {
        let mut hasher = <Self::Node as TNode>::Hash::new_hasher();

        self.head.hash(&mut hasher);
        self.cells.iter().for_each(|cell| cell.hash(&mut hasher));

        hasher.finalize()
    }
}

#[derive(Default)]
pub struct BranchCell<'a, Node>(Node::Key, NodeRef<'a, Node>)
where Node: TNode;

impl<'a, Node: TNode> crate::hash::traits::Hashable for BranchCell<'a, Node>
{
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.0.hash(hasher);
        self.1.id().hash(hasher);
    }
}

impl<'a, Node: TNode> std::cmp::PartialOrd<Node::Key> for BranchCell<'a, Node>
{
    fn partial_cmp(&self, other: &Node::Key) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<'a, Node: TNode>  std::cmp::PartialOrd<&Node::Key> for &mut BranchCell<'a, Node>
{
    fn partial_cmp(&self, other: &&Node::Key) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<'a, Node: TNode>  std::cmp::PartialEq<Node::Key> for BranchCell<'a, Node>
{
    fn eq(&self, other: &Node::Key) -> bool {
        self.0 == *other
    }
}
