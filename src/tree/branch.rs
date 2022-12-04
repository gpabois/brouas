use self::traits::Branch as TraitBranch;
use super::cells::branch::BranchCells;
use super::cells::branch::traits::BranchCells as TraitBranchCells;
use super::node::traits::Node as TNode;
use super::node_ref::NodeRef;

pub mod traits {
    use crate::tree::node::traits::Node;
    use crate::tree::node_ref::NodeRef;
    
    /// A branch of a Merkle B+ Tree
    pub trait Branch
    {
        type Node: Node;

        /// Create a branch
        fn new(left: NodeRef<Self::Node>, key: <Self::Node as Node>::Key, right: NodeRef<Self::Node>) -> Self;

        /// Insert a cell into the branch
        fn insert(&mut self, left: NodeRef<Self::Node>, key: <Self::Node as Node>::Key, right: NodeRef<Self::Node>);
        
        /// Search the node satifying the key
        fn search_node<'a>(&'a self, key: &<Self::Node as Node>::Key) -> &'a NodeRef<Self::Node>;

        /// Split the branch, and returns right node
        fn split(&mut self) -> (Self, <Self::Node as Node>::Key, Self) where Self: Sized;

        /// Returns the children refs
        fn children<'a>(&'a self) -> Vec<&'a NodeRef<Self::Node>>;

        /// Compute the hash
        fn compute_hash(&self) -> <Self::Node as Node>::Hash;

        /// 
        fn is_full(&self) -> bool;
    }
     
}

#[derive(Clone)]
pub struct Branch<'a, Node>
where Node: crate::tree::node::traits::Node
{
    cells: BranchCells<'a, Node>
}

impl<'a, Node> Branch<'a, Node>
where Node: crate::tree::node::traits::Node
{
    fn new_from_cells(cells: BranchCells<'a, Node>) -> Self {
        Self {cells: cells}
    } 
}

impl<'a, Node> TraitBranch for Branch<'a, Node>
where Node: crate::tree::node::traits::Node
{
    type Node = Node;

    fn new(left: NodeRef<'a, Self::Node>, key: <Self::Node as TNode>::Key, right: NodeRef<'a, Node>) -> Self 
    {
        Self {
            cells: BranchCells::new(left, key, right)
        }
    }

    fn insert(&mut self, left: NodeRef<'a, Node>, key: <Self::Node as TNode>::Key, right: NodeRef<'a, Node>) 
    {
        self.cells.insert(left, key, right)
    }

    /// Search the node satifying the key
    fn search_node<'b>(&'b self, key: &<Self::Node as TNode>::Key) -> &'b NodeRef<'a, Self::Node>
    {
        self.cells.search(key)
    }

    /// Split the branch, and returns right node
    fn split_branch(&mut self) -> (<Self::Node as TNode>::Key, Self) where Self: Sized
    {
        let (key, right_cells) = self.cells.split();
        (key, Self::new_from_cells(right_cells))
    }

    fn children<'b>(&'a self) -> Vec<&'b NodeRef<'a, Self::Node>> 
    {
        self.cells.children()
    }

    fn compute_hash(&self) -> <Self::Node as TNode>::Hash 
    {
        self.cells.compute_hash()
    }

    fn is_full(&self) -> bool {
        self.cells.is_full()
    }
}