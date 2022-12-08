use crate::tree::cells::leaf::{LeafCells, LeafCell};
use super::cells::leaf::traits::LeafCells as TraitLeafCells;
use crate::tree::node::traits::Node as TNode;
use self::traits::Leaf as TLeaf;

pub mod traits 
{
    use crate::tree::node::traits::Node as TNode;

    pub trait Leaf
    {
        type Node: TNode;

        /// Create a new leaf
        fn new(key: <Self::Node as TNode>::Key, element: <Self::Node as TNode>::Element) -> Self;
     
        /// Search element behind key
        fn search(&self, key: &<Self::Node as TNode>::Key) -> Option<&<Self::Node as TNode>::Element>;
        fn search_mut(&mut self, key: &<Self::Node as TNode>::Key) -> Option<&mut <Self::Node as TNode>::Element>;

        // Check if the leaf is full
        fn is_full(&self) -> bool;
    
        // Split the leaf
        fn split(&mut self) -> (Self, <Self::Node as TNode>::Key, Self) where Self: Sized;
    
        // Insert cell
        fn insert(&mut self, key: <Self::Node as TNode>::Key, element: <Self::Node as TNode>::Element);
        
        /// Compute the hash of the leaf
        fn compute_hash(&self) -> <Self::Node as TNode>::Hash;
    }   
}

pub struct Leaf< Node>
where Node: TNode
{
    cells: LeafCells< Node>
}

impl< Node> Leaf< Node>
where Node: TNode
{
    fn from_cells(cells: impl Into<LeafCells< Node>>) -> Self {
        Self {
            cells: cells.into()
        }
    }
}

impl<Node> ToOwned for Leaf<Node>
where Node: TNode
{
    type Owned = Self;

    fn to_owned(&self) -> Self::Owned {
        Self {
            cells: self.cells.to_owned()
        }
    }
}

impl< Node> TLeaf for Leaf< Node>
where Node: TNode {
    type Node = Node;

    fn new(key: Node::Key, element: Node::Element) -> Self {
        Self {
            cells: LeafCells::new(LeafCell::new(key, element))
        }
    }

    /// Search element behind key
    fn search(&self, key: &Node::Key) -> Option<&Node::Element>
    {
        self.cells.search(key)
    }
    fn search_mut(&mut self, key: &<Self::Node as TNode>::Key) -> Option<&mut <Self::Node as crate::tree::node::traits::Node>::Element> {
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

    fn compute_hash(&self) -> <Self::Node as TNode>::Hash {
        TraitLeafCells::compute_hash(&self.cells)
    }


}