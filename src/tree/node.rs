use super::node_ref::WeakNode;
use super::{Leaf, Branch};
use super::leaf::traits::Leaf as TLeaf;
use super::branch::traits::Branch as TBranch;

#[derive(Clone)]
pub enum NodeType<Branch, Leaf>
{
    Leaf(Leaf),
    Branch(Branch)
}

impl<Branch, Leaf> NodeType<Branch, Leaf>
{
    pub fn as_branch(&self) -> Option<&Branch>
    {
        match self {
            NodeType::Branch(branch) => Some(branch),
            _ => None
        }
    }
    pub fn as_leaf(&self) -> Option<&Leaf>
    {
        match self {
            NodeType::Leaf(leaf) => Some(leaf),
            _ => None
        }
    }

    pub fn as_mut_branch(&mut self) -> Option<&mut Branch>
    {
        match self {
            NodeType::Branch(branch) => Some(branch),
            _ => None
        }
    }
    pub fn as_mut_leaf(&mut self) -> Option<&mut Leaf>
    {
        match self {
            NodeType::Leaf(leaf) => Some(leaf),
            _ => None
        }
    }
}

pub mod traits {
    use crate::tree::{node_ref::WeakNode, nodes::traits::Nodes as TNodes};

    use super::NodeType;

    /// The MBT Node Trait
    pub trait Node<'a>: From<Self::Leaf> + From<Self::Branch> + PartialEq<Self::Hash>
    {
        const SIZE: usize;

        type Key: Clone + PartialOrd + PartialEq + Ord + crate::hash::traits::Hashable;
        type Hash: Clone + PartialEq + std::fmt::Display + Default + crate::hash::traits::Hash + crate::hash::traits::Hashable;
        type Element: Clone + crate::hash::traits::Hashable;

        type Leaf: crate::tree::leaf::traits::Leaf<'a, Node=Self>;
        type Branch: crate::tree::branch::traits::Branch<'a, Node=Self>;

        fn r#as(&'a self) -> &'a NodeType<Self::Branch, Self::Leaf>;
        fn as_mut(&'a mut self) -> &'a mut NodeType<Self::Branch, Self::Leaf>;

        /// Get the children of the node
        fn children(&'a self) -> Vec<&'a WeakNode<'a, Self>>;

        /// Compute the hash of the node
        fn calculate_hash_if_changed<Nodes: TNodes<'a, Node=Self>>(&mut self, nodes: &mut Nodes) -> Self::Hash;
        
        fn set_hash(&mut self, hash: Self::Hash);
        fn get_hash(&self) -> Option<Self::Hash>;

        /// Split the node
        fn split(&mut self) -> (Self, Self::Key, Self);
        
        /// The node is full ?
        fn is_full(&'a self) -> bool;
    }

}

pub struct Node<'a, const SIZE: usize, Hash, Key, Element>
where   Hash: Clone + PartialEq + std::fmt::Display + crate::hash::traits::Hash + crate::hash::traits::Hashable + Default,
        Key: PartialEq + PartialOrd + Ord + Clone + crate::hash::traits::Hashable,
        Element: Clone + crate::hash::traits::Hashable 
{
    node_type: NodeType<Branch<'a, Self>, Leaf<'a, Self>>,
    hash: Option<Hash>
}

impl<'a, const SIZE: usize, Hash, Key, Element> From<Branch<'a, Self>> for Node<'a, SIZE, Hash, Key, Element>
where   Hash: Clone + PartialEq + std::fmt::Display + crate::hash::traits::Hash + crate::hash::traits::Hashable + Default,
        Key: PartialEq + PartialOrd + Ord + Clone + crate::hash::traits::Hashable ,
        Element: Clone + crate::hash::traits::Hashable 
{
    fn from(branch: Branch<'a, Self>) -> Self {
        Self {
            node_type: NodeType::Branch(branch),
            hash: None
        }
    }
}

impl<'a, const SIZE: usize, Hash, Key, Element> From<Leaf<'a, Self>> for Node<'a, SIZE, Hash, Key, Element>
where   Hash: Clone + PartialEq + std::fmt::Display + crate::hash::traits::Hash + crate::hash::traits::Hashable + Default,
        Key: PartialEq + PartialOrd + Ord + Clone + crate::hash::traits::Hashable ,
        Element: Clone + crate::hash::traits::Hashable 
{
    fn from(leaf: Leaf<'a, Self>) -> Self {
        Self {
            node_type: NodeType::Leaf(leaf),
            hash: None
        }
    }
}

impl<'a, const SIZE: usize, Hash, Key, Element> PartialEq<Hash> for Node<'a, SIZE, Hash, Key, Element>
where   Hash: Clone + PartialEq + std::fmt::Display + crate::hash::traits::Hash + crate::hash::traits::Hashable + Default,
        Key: PartialEq + PartialOrd + Ord + Clone + crate::hash::traits::Hashable ,
        Element: Clone + crate::hash::traits::Hashable 
{
    fn eq(&self, other: &Hash) -> bool {
        self.hash.eq(&Some(other.clone()))
    }
}

impl<'a, const SIZE: usize, Hash, Key, Element> self::traits::Node<'a> for Node<'a, SIZE, Hash, Key, Element>
where   Hash: Clone + PartialEq + std::fmt::Display + Default + crate::hash::traits::Hash + crate::hash::traits::Hashable,
        Key: PartialEq + PartialOrd + Ord + Clone + crate::hash::traits::Hashable ,
        Element: Clone + crate::hash::traits::Hashable 
{
    const SIZE: usize = SIZE;

    type Key        = Key;
    type Hash       = Hash;
    type Element    = Element;

    type Leaf       = Leaf<'a, Self>;
    type Branch     = Branch<'a, Self>;

    fn r#as(&self) -> &NodeType<Self::Branch, Self::Leaf> {
        &self.node_type
    }

    fn as_mut(&mut self) -> &mut NodeType<Self::Branch, Self::Leaf> {
       &mut self.node_type
    }

    fn children(&'a self) -> Vec<&'a WeakNode<'a, Self>> {
        match self.r#as() {
            NodeType::Branch(branch) => branch.children(),
            _ => vec![]
        }
    }

    fn calculate_hash_if_changed<Nodes: super::nodes::traits::Nodes<'a, Node=Self>>(&mut self, nodes: &mut Nodes) -> Self::Hash {
        let hash = match self.r#as() {
            NodeType::Branch(branch) => branch.compute_hash(),
            NodeType::Leaf(leaf) => leaf.compute_hash()
        };

        self.hash = Some(hash.clone());
        hash        
    }

    fn set_hash(&mut self, hash: Self::Hash) {
        self.hash = Some(hash)
    }

    fn get_hash(&self) -> Option<Self::Hash> {
        return self.hash.clone()
    }

    fn split(&mut self) -> (Self, Self::Key, Self) {
        match self.as_mut() {
            NodeType::Branch(branch) => {
                let (left_branch, key, right_branch) = branch.split();
                let right_node = Self::from(right_branch);
                let left_node = Self::from(left_branch);
                (left_node, key, right_node)
            },
            NodeType::Leaf(leaf) => {
                let (left_leaf, key, right_leaf) = leaf.split();
                let right_node = Self::from(right_leaf);
                let left_node = Self::from(left_leaf);
                (left_node, key, right_node)
            }
        }
    }

    fn is_full(&'a self) -> bool {
        match self.r#as() {
            NodeType::Branch(branch) => branch.is_full(),
            NodeType::Leaf(leaf) => leaf.is_full()
        }
    }
}