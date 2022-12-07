use self::traits::Branch as TraitBranch;
use super::cells::branch::BranchCells;
use super::cells::branch::traits::BranchCells as TraitBranchCells;
use super::node::traits::Node as TNode;
use super::nodes::traits::Nodes as TNodes;
use super::node_ref::WeakNode;
use super::result::TreeResult;

pub mod traits {
    use crate::tree::node::traits::Node;
    use crate::tree::nodes::traits::Nodes as TNodes;
    use crate::tree::node_ref::WeakNode;
    use crate::tree::result::TreeResult;
    
    /// A branch of a Merkle B+ Tree
    pub trait Branch
    {
        type Node: Node;

        /// Create a branch
        fn new(left: WeakNode< Self::Node>, key: <Self::Node as Node>::Key, right: WeakNode< Self::Node>) -> Self;

        /// Insert a cell into the branch
        fn insert(&mut self, place: &WeakNode< Self::Node>, left: WeakNode< Self::Node>, key: <Self::Node as Node>::Key, right: WeakNode< Self::Node>);
        
        /// Search the node satifying the key
        fn search_node(&self, key: &<Self::Node as Node>::Key) -> &WeakNode<Self::Node>;

        /// Split the branch, and returns right node
        fn split(&mut self) -> (Self, <Self::Node as Node>::Key, Self) where Self: Sized;

        /// Returns the children refs
        fn children(&self) -> Vec<&WeakNode< Self::Node>>;

        /// Compute the hash
        fn compute_hash<Nodes: TNodes< Node=Self::Node>>(&self, nodes: &Nodes) -> TreeResult<<Self::Node as Node>::Hash, Self::Node>;

        /// 
        fn is_full(&self) -> bool;
    }
     
}

pub struct Branch< Node>
where Node: TNode
{
    cells: BranchCells< Node>
}

impl< Node> Branch< Node>
where Node: TNode
{
    fn new_from_cells(cells: BranchCells< Node>) -> Self {
        Self {cells: cells}
    } 
}

impl< Node> TraitBranch for Branch< Node>
where Node: TNode
{
    type Node = Node;

    fn new(left: WeakNode< Self::Node>, key: <Self::Node as TNode>::Key, right: WeakNode< Node>) -> Self 
    {
        Self {
            cells: BranchCells::new(left, key, right)
        }
    }

    fn insert(&mut self, place: &WeakNode< Node>, left: WeakNode< Node>, key: <Self::Node as TNode>::Key, right: WeakNode< Node>) 
    {
        self.cells.insert(place, left, key, right)
    }

    /// Search the node satifying the key
    fn search_node(&self, key: &<Self::Node as TNode>::Key) -> &WeakNode< Self::Node>
    {
        self.cells.search(key)
    }

    /// Split the branch, and returns right node
    fn split(&mut self) -> (Self, <Self::Node as TNode>::Key, Self) where Self: Sized
    {
        let (left_cells, key, right_cells) = self.cells.split();
        (
            Self::new_from_cells(left_cells),
            key, 
            Self::new_from_cells(right_cells)
        )
    }

    fn children(&self) -> Vec<&WeakNode< Self::Node>> 
    {
        self.cells.nodes()
    }

    fn compute_hash<Nodes: TNodes< Node=Self::Node>>(&self, nodes: &Nodes) -> TreeResult<<Self::Node as TNode>::Hash, Self::Node>
    {
        self.cells.compute_hash(nodes)
    }

    fn is_full(&self) -> bool {
        self.cells.is_full()
    }

}