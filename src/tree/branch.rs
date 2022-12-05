use self::traits::Branch as TraitBranch;
use super::cells::branch::BranchCells;
use super::cells::branch::traits::BranchCells as TraitBranchCells;
use super::node::traits::Node as TNode;
use super::node_ref::WeakNode;

pub mod traits {
    use crate::tree::node::traits::Node;
    use crate::tree::node_ref::WeakNode;
    
    /// A branch of a Merkle B+ Tree
    pub trait Branch<'a>
    {
        type Node: Node<'a>;

        /// Create a branch
        fn new(left: WeakNode<'a, Self::Node>, key: <Self::Node as Node<'a>>::Key, right: WeakNode<'a, Self::Node>) -> Self;

        /// Insert a cell into the branch
        fn insert(&'a mut self, place: &WeakNode<'a, Self::Node>, left: WeakNode<'a, Self::Node>, key: <Self::Node as Node<'a>>::Key, right: WeakNode<'a, Self::Node>);
        
        /// Search the node satifying the key
        fn search_node(&'a self, key: &<Self::Node as Node<'a>>::Key) -> &'a WeakNode<Self::Node>;
        fn search_mut_node(&'a mut self, key: &<Self::Node as Node<'a>>::Key) -> &'a mut WeakNode<Self::Node>;

        /// Split the branch, and returns right node
        fn split(&mut self) -> (Self, <Self::Node as Node<'a>>::Key, Self) where Self: Sized;

        /// Returns the children refs
        fn children(&'a self) -> Vec<&'a WeakNode<'a, Self::Node>>;

        /// Compute the hash
        fn compute_hash(&self) -> <Self::Node as Node<'a>>::Hash;

        /// 
        fn is_full(&self) -> bool;
    }
     
}

pub struct Branch<'a, Node>
where Node: TNode<'a>
{
    cells: BranchCells<'a, Node>
}

impl<'a, Node> Branch<'a, Node>
where Node: TNode<'a>
{
    fn new_from_cells(cells: BranchCells<'a, Node>) -> Self {
        Self {cells: cells}
    } 
}

impl<'a, Node> TraitBranch<'a> for Branch<'a, Node>
where Node: TNode<'a>
{
    type Node = Node;

    fn new(left: WeakNode<'a, Self::Node>, key: <Self::Node as TNode<'a>>::Key, right: WeakNode<'a, Node>) -> Self 
    {
        Self {
            cells: BranchCells::new(left, key, right)
        }
    }

    fn insert(&'a mut self, place: &WeakNode<'a, Node>, left: WeakNode<'a, Node>, key: <Self::Node as TNode<'a>>::Key, right: WeakNode<'a, Node>) 
    {
        self.cells.insert(place, left, key, right)
    }

    /// Search the node satifying the key
    fn search_node(&'a self, key: &<Self::Node as TNode<'a>>::Key) -> &'a WeakNode<'a, Self::Node>
    {
        self.cells.search(key)
    }

    /// Split the branch, and returns right node
    fn split(&mut self) -> (Self, <Self::Node as TNode<'a>>::Key, Self) where Self: Sized
    {
        let (left_cells, key, right_cells) = self.cells.split();
        (
            Self::new_from_cells(left_cells),
            key, 
            Self::new_from_cells(right_cells)
        )
    }

    fn children(&'a self) -> Vec<&'a WeakNode<'a, Self::Node>> 
    {
        self.cells.nodes()
    }

    fn compute_hash(&self) -> <Self::Node as TNode<'a>>::Hash 
    {
        self.cells.compute_hash()
    }

    fn is_full(&self) -> bool {
        self.cells.is_full()
    }

    fn search_mut_node(&'a mut self, key: &<Self::Node as TNode<'a>>::Key) -> &'a mut WeakNode<Self::Node> {
        self.cells.search_mut(key)
    }
}