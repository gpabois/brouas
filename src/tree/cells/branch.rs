use self::traits::BranchCells as TraitBranchCells;
use crate::tree::node::traits::Node as TNode;
use crate::{hash::traits::{Hashable, Hasher}, tree::node_ref::WeakNode};

pub mod traits {
    use crate::tree::node_ref::WeakNode;
    use crate::tree::node::traits::Node as TNode;

    pub trait BranchCells<'a>
    {
        type Node: TNode<'a>;
        
        /// New branch cells
        fn new(left: WeakNode<'a, Self::Node>, key: <Self::Node as TNode<'a>>::Key, right: WeakNode<'a, Self::Node>) -> Self;
        /// Search the node based on the key
        fn search(&'a self, k: &<Self::Node as TNode<'a>>::Key) -> &'a WeakNode<'a, Self::Node>;
        /// Split the cells
        fn split(&mut self) -> (Self, <Self::Node as TNode<'a>>::Key, Self) where Self: Sized;
        /// The cells are full
        fn is_full(&self) -> bool;
        /// Insert a cell
        fn insert(&'a mut self, place: &WeakNode<'a, Self::Node>, left: WeakNode<'a, Self::Node>, key: <Self::Node as TNode<'a>>::Key, right: WeakNode<'a, Self::Node>);
        /// Compute the branch cells hash
        fn compute_hash(&self) -> <Self::Node as TNode<'a>>::Hash;
        /// Return the weak nodes references
        fn nodes(&'a self) -> Vec<&'a WeakNode<'a, Self::Node>>;
        /// Return the mutable weak nodes references
        fn mut_nodes(&'a mut self) -> Vec<&'a mut WeakNode<'a, Self::Node>>;
    }
}

pub struct BranchCells<'a, Node>
where Node: TNode<'a>
{
    head: WeakNode<'a, Node>,
    cells: Vec<BranchCell<'a, Node>>
} 

impl<'a, Node> TraitBranchCells<'a> for BranchCells<'a, Node>
where Node: TNode<'a>
{
    type Node = Node;

    fn new(left: WeakNode<'a, Self::Node>, key: <Self::Node as TNode<'a>>::Key, right: WeakNode<'a, Self::Node>) -> Self {
        Self {
            head: left,
            cells: vec![BranchCell(key, right)]
        }
    }

    fn search(&'a self, k: &<Self::Node as TNode<'a>>::Key) -> &'a WeakNode<'a, Self::Node> {
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

    fn split(&mut self) -> (Self, <Self::Node as TNode<'a>>::Key, Self) {
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

    fn insert(&'a mut self, place: &WeakNode<'a, Self::Node>, left: WeakNode<'a, Self::Node>, key: <Self::Node as TNode<'a>>::Key, right: WeakNode<'a, Self::Node>) 
    {
        let (idx, cell) = self.cells.iter_mut().enumerate().find(|(idx, cell)| std::ptr::eq(place, &cell.1)).unwrap();
        cell.1 = left;
        self.cells.insert(idx + 1, BranchCell(key, right));
    }

    fn compute_hash(&self) -> <Self::Node as TNode<'a>>::Hash {
        todo!();
        /*
        let mut hasher = <Self::Node as TNode>::Hash::new_hasher();

        self.head.hash(&mut hasher);
        self.cells.iter().for_each(|cell| cell.hash(&mut hasher));

        hasher.finalize()
        */
    }

    fn nodes(&'a self) -> Vec<&'a WeakNode<'a, Self::Node>> {
        todo!()
    }

    fn mut_nodes(&'a mut self) -> Vec<&'a mut WeakNode<'a, Self::Node>> {
        todo!()
    }
}

#[derive(Default)]
pub struct BranchCell<'a, Node>(Node::Key, WeakNode<'a, Node>)
where Node: TNode<'a>;

impl<'a, Node: TNode<'a>> crate::hash::traits::Hashable for BranchCell<'a, Node>
{
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.0.hash(hasher);
        self.1.id().hash(hasher);
    }
}

impl<'a, Node: TNode<'a>> std::cmp::PartialOrd<Node::Key> for BranchCell<'a, Node>
{
    fn partial_cmp(&self, other: &Node::Key) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<'a, Node: TNode<'a>>  std::cmp::PartialOrd<&Node::Key> for &mut BranchCell<'a, Node>
{
    fn partial_cmp(&self, other: &&Node::Key) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<'a, Node: TNode<'a>>  std::cmp::PartialEq<Node::Key> for BranchCell<'a, Node>
{
    fn eq(&self, other: &Node::Key) -> bool {
        self.0 == *other
    }
}
