use crate::tree::cells::leaf::{LeafCells, LeafCell};
use super::cells::leaf::traits::LeafCells as TraitLeafCells;

pub mod traits 
{
    pub trait Leaf
    {
        type Node: crate::tree::node::traits::Node;

        /// Create a new leaf
        fn new(key: <Self::Node as crate::tree::node::traits::Node>::Key, element: <Self::Node as crate::tree::node::traits::Node>::Element) -> Self;
     
        /// Search element behind key
        fn search<'a>(&'a self, key: &<Self::Node as crate::tree::node::traits::Node>::Key) -> Option<&'a <Self::Node as crate::tree::node::traits::Node>::Element>;
        fn search_mut<'a>(&'a mut self, key: &<Self::Node as crate::tree::node::traits::Node>::Key) -> Option<&'a mut <Self::Node as crate::tree::node::traits::Node>::Element>;

        // Check if the leaf is full
        fn is_full(&self) -> bool;
    
        // Split the leaf
        fn split_leaf(&mut self) -> (<Self::Node as crate::tree::node::traits::Node>::Key, Self) where Self: Sized;
    
        // Insert cell
        fn insert(&mut self, key: <Self::Node as crate::tree::node::traits::Node>::Key, element: <Self::Node as crate::tree::node::traits::Node>::Element);
        
        /// Compute the hash of the leaf
        fn compute_hash(&self) -> <Self::Node as crate::tree::node::traits::Node>::Hash;
    }   
}


#[derive(Clone)]
pub struct Leaf<Node>
where Node: crate::tree::node::traits::Node
{
    cells: LeafCells<Node>
}

impl<Node> Leaf<Node>
where Node: crate::tree::node::traits::Node
{
    fn from_cells(cells: impl Into<LeafCells<Node>>) -> Self {
        Self {
            cells: cells.into()
        }
    }
}

impl<Node> self::traits::Leaf for Leaf<Node>
where Node: crate::tree::node::traits::Node {
    type Node = Node;

    fn new(key: Node::Key, element: Node::Element) -> Self {
        Self {
            cells: LeafCells::new(LeafCell::new(key, element))
        }
    }

    /// Search element behind key
    fn search<'a>(&'a self, key: &Node::Key) -> Option<&'a Node::Element>
    {
        self.cells.search(key)
    }
    fn search_mut<'a>(&'a mut self, key: &<Self::Node as crate::tree::node::traits::Node>::Key) -> Option<&'a mut <Self::Node as crate::tree::node::traits::Node>::Element> {
        self.cells.search_mut(key)
    }
    // 
    fn is_full(&self) -> bool
    {
        return self.cells.is_full();
    }

    // Split the leaf
    fn split_leaf(&mut self) -> (Node::Key, Self) where Self: Sized
    {
        let (key, right_cells) = self.cells.split();
        (key, Self::from_cells(right_cells))
    }

    // Insert cell
    fn insert(&mut self, key: Node::Key, element: Node::Element)
    {
        self.cells.insert(key, element);
    }

    fn compute_hash(&self) -> <Self::Node as crate::tree::node::traits::Node>::Hash {
        todo!()
    }


}