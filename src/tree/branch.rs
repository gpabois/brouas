use super::{Node, cells::branch::BranchCells, NodeRef};

///
/// A branch of a Merkle B+ Tree
/// *1 [1] *2 [2] *3
pub trait Branch<const SIZE: usize>
{
    type Hash: Clone + PartialEq;
    type Key: PartialOrd;
    type Node: Node<SIZE>;
    type Cells: BranchCells<SIZE, Key=Self::Key, Hash=Self::Hash>;

    fn new_from_cells(cells: Self::Cells) -> Self;
    fn new(left: NodeRef<Self::Hash>, key: Self::Key, right: NodeRef<Self::Hash>) -> Self;

    /// To implement
    fn borrow_cells<'a>(&'a self) -> &'a Self::Cells;
    fn borrow_mut_cells<'a>(&'a self) -> &'a mut Self::Cells;

    fn insert(&mut self, left: NodeRef<Self::Hash>, key: Self::Key, right: NodeRef<Self::Hash>);
 
    fn search_node<'a>(&'a self, key: &Self::Key) -> &'a NodeRef<Self::Hash>
    {
        self.borrow_cells().search(key)
    }

    /// Split the branch, and returns right node
    fn split_branch(&mut self) -> (Self::Key, Self) where Self: Sized
    {
        let (key, right_cells) = self.borrow_mut_cells().split();
        (key, Self::new_from_cells(right_cells))
    }

    /// Returns the children refs
    fn children_ref<'a>(&'a self) -> Vec<&'a NodeRef<Self::Hash>>;

    /// Compute the hash
    fn compute_hash<Arena: crate::arena::Arena<Key=Self::Hash, Value=Self::Node>>(&self, arena: &Arena) -> Self::Hash;

}
