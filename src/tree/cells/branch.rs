use crate::tree::{NodeRef, node::traits::{Node, BorrowNode}};
use self::traits::BranchCells as TraitBranchCells;
use crate::hash::traits::{Hash, Hasher, Hashable};

pub mod traits {
    use crate::tree::{NodeRef, node::traits::BorrowNode};

    pub trait BranchCells
    {
        type Node: crate::tree::node::traits::Node;
        
        /// New branch cells
        fn new(left: NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>, key: <Self::Node as crate::tree::node::traits::Node>::Key, right: NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>) -> Self;
        /// Search the node based on the key
        fn search<'a>(&'a self, k: &<Self::Node as crate::tree::node::traits::Node>::Key) -> &'a NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>;
        /// Split the cells
        fn split(&mut self) -> (<Self::Node as crate::tree::node::traits::Node>::Key, Self);
        /// The cells are full
        fn is_full(&self) -> bool;
        /// Insert a cell
        fn insert(&mut self, left: NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>, key: <Self::Node as crate::tree::node::traits::Node>::Key, right: NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>);
        /// Compute the branch cells hash
        fn compute_hash<Nodes: BorrowNode<Self::Node>>(&self, nodes: &Nodes) -> <Self::Node as crate::tree::node::traits::Node>::Hash;
    }
}

#[derive(Clone)]
pub struct BranchCells<Node>
where Node: crate::tree::node::traits::Node
{
    head: NodeRef<Node::Hash>,
    cells: Vec<BranchCell<Node>>
} 

impl<Node> TraitBranchCells for BranchCells<Node>
where Node: crate::tree::node::traits::Node
{
    type Node = Node;
    fn search<'a>(&'a self, k: &<Self::Node as crate::tree::node::traits::Node>::Key) -> &'a NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>
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
 

    fn split(&mut self) -> (<Self::Node as crate::tree::node::traits::Node>::Key, Self) {
        let middle_index = <Self::Node as crate::tree::node::traits::Node>::SIZE/2;
        let lefts = &self.cells[0..middle_index - 1];
        let rights = &self.cells[middle_index + 1..];
        let middle_cell = self.cells[middle_index].clone();

        let middle_key = middle_cell.0;
        let right_cell = Self {
            head: middle_cell.1,
            cells: rights.iter().cloned().collect()
        };

        self.cells = lefts.iter().cloned().collect();

        return (middle_key, right_cell)
    }

    fn is_full(&self) -> bool {
        self.cells.len() >= <Self::Node as crate::tree::node::traits::Node>::SIZE
    }

    fn insert(&mut self, left: NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>, key: <Self::Node as crate::tree::node::traits::Node>::Key, right: NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>) {
        let (idx, cell) = self.cells
        .iter_mut()
        .enumerate()
        .find(|(_idx, cell)| cell.1 == left)
        .expect("Expecting to find node ref in tree branch");

        let right_key = cell.0.clone();
        cell.0 = key;

        self.cells.insert(idx + 1, BranchCell(right_key, right));
    }

    fn new(left: NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>, key: <Self::Node as crate::tree::node::traits::Node>::Key, right: NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>) -> Self {
        Self {
            head: left,
            cells: vec![BranchCell(key, right)]
        }
    }

    fn compute_hash<Nodes: BorrowNode<Self::Node>>(&self, nodes: &Nodes) -> <Self::Node as crate::tree::node::traits::Node>::Hash {
        let mut hasher = <Self::Node as crate::tree::node::traits::Node>::Hash::new_hasher();

        self.head.hash(&mut hasher);
        self.cells.iter().for_each(|cell| cell.hash(&mut hasher));

        hasher.finalize()
    }
}

pub struct BranchCell<Node: crate::tree::node::traits::Node>(Node::Key, NodeRef<Node::Hash>);

impl<Node: crate::tree::node::traits::Node> Clone for BranchCell<Node>
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<Node: crate::tree::node::traits::Node> crate::hash::traits::Hashable for BranchCell<Node>
{
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.0.hash(hasher);
        self.1.hash(hasher);
    }
}

impl<Node: crate::tree::node::traits::Node> std::cmp::PartialOrd<Node::Key> for BranchCell<Node>
{
    fn partial_cmp(&self, other: &Node::Key) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<Node: crate::tree::node::traits::Node>  std::cmp::PartialOrd<&Node::Key> for &mut BranchCell<Node>
{
    fn partial_cmp(&self, other: &&Node::Key) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<Node: crate::tree::node::traits::Node>  std::cmp::PartialEq<Node::Key> for BranchCell<Node>
{
    fn eq(&self, other: &Node::Key) -> bool {
        self.0 == *other
    }
}
