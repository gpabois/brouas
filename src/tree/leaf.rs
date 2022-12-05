use crate::tree::cells::leaf::{LeafCells, LeafCell};
use super::cells::leaf::traits::LeafCells as TraitLeafCells;
use crate::tree::node::traits::Node as TNode;
use self::traits::Leaf as TLeaf;

pub mod traits 
{
    use crate::tree::node::traits::Node as TNode;

    pub trait Leaf<'a>
    {
        type Node: TNode<'a>;

        /// Create a new leaf
        fn new(key: <Self::Node as TNode<'a>>::Key, element: <Self::Node as TNode<'a>>::Element) -> Self;
     
        /// Search element behind key
        fn search(&'a self, key: &<Self::Node as TNode<'a>>::Key) -> Option<&'a <Self::Node as TNode<'a>>::Element>;
        fn search_mut(&'a mut self, key: &<Self::Node as TNode<'a>>::Key) -> Option<&'a mut <Self::Node as TNode<'a>>::Element>;

        // Check if the leaf is full
        fn is_full(&self) -> bool;
    
        // Split the leaf
        fn split(&mut self) -> (Self, <Self::Node as TNode<'a>>::Key, Self) where Self: Sized;
    
        // Insert cell
        fn insert(&mut self, key: <Self::Node as TNode<'a>>::Key, element: <Self::Node as TNode<'a>>::Element);
        
        /// Compute the hash of the leaf
        fn compute_hash(&self) -> <Self::Node as TNode<'a>>::Hash;
    }   
}

pub struct Leaf<'a, Node>
where Node: TNode<'a>
{
    cells: LeafCells<'a, Node>
}

impl<'a, Node> Leaf<'a, Node>
where Node: TNode<'a>
{
    fn from_cells(cells: impl Into<LeafCells<'a, Node>>) -> Self {
        Self {
            cells: cells.into()
        }
    }
}

impl<'a, Node> TLeaf<'a> for Leaf<'a, Node>
where Node: TNode<'a> {
    type Node = Node;

    fn new(key: Node::Key, element: Node::Element) -> Self {
        Self {
            cells: LeafCells::new(LeafCell::new(key, element))
        }
    }

    /// Search element behind key
    fn search(&'a self, key: &Node::Key) -> Option<&'a Node::Element>
    {
        self.cells.search(key)
    }
    fn search_mut(&'a mut self, key: &<Self::Node as TNode<'a>>::Key) -> Option<&'a mut <Self::Node as crate::tree::node::traits::Node>::Element> {
        self.cells.search_mut(key)
    }
    // 
    fn is_full(&self) -> bool
    {
        return self.cells.is_full();
    }

    // Split the leaf
    fn split(&mut self) -> (Self, Node::Key, Self) where Self: Sized
    {
        let (left_cells, key, right_cells) = self.cells.split();
        (Self::from_cells(left_cells), key, Self::from_cells(right_cells))
    }

    // Insert cell
    fn insert(&mut self, key: Node::Key, element: Node::Element)
    {
        self.cells.insert(key, element);
    }

    fn compute_hash(&self) -> <Self::Node as crate::tree::node::traits::Node<'a>>::Hash {
        self.cells.compute_hash()
    }


}