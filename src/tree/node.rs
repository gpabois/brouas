use super::node_ref::WeakNode;
use super::result::TreeResult;
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
    use crate::tree::{node_ref::WeakNode, nodes::traits::Nodes as TNodes, result::TreeResult};

    use super::NodeType;

    /// The MBT Node Trait
    pub trait Node: From<Self::Leaf> + From<Self::Branch> + PartialEq<Self::Hash>
    {
        const SIZE: usize;

        type Key: Clone + PartialOrd + PartialEq + Ord + crate::hash::traits::Hashable;
        type Hash: Copy + Clone + PartialEq + std::fmt::Display + Default + crate::hash::traits::Hash + crate::hash::traits::Hashable;
        type Element: Clone + crate::hash::traits::Hashable;

        type Leaf: crate::tree::leaf::traits::Leaf<Node=Self>;
        type Branch: crate::tree::branch::traits::Branch<Node=Self>;

        fn r#as(&self) -> &NodeType<Self::Branch, Self::Leaf>;
        fn as_mut(&mut self) -> &mut NodeType<Self::Branch, Self::Leaf>;

        /// Get the children of the node
        fn children(&self) -> Vec<&WeakNode< Self>>;
        /// Compute the hash of the node
        fn compute_hash<Nodes: TNodes< Node=Self>>(&self, nodes: &Nodes) -> TreeResult<Self::Hash, Self>;
        /// Set the hash
        fn set_hash(&mut self, hash: Self::Hash);
        /// Get the hash, if not invalid
        fn get_hash(&self) -> Option<Self::Hash>;
        /// Split the node
        fn split(&mut self) -> (Self, Self::Key, Self);
        /// The node is full ?
        fn is_full(&self) -> bool;
    }

}

pub struct Node< const SIZE: usize, Hash, Key, Element>
where   Hash: Copy + Clone + PartialEq + std::fmt::Display + crate::hash::traits::Hash + crate::hash::traits::Hashable + Default,
        Key: PartialEq + PartialOrd + Ord + Clone + crate::hash::traits::Hashable,
        Element: Clone + crate::hash::traits::Hashable 
{
    node_type: NodeType<Branch< Self>, Leaf<Self>>,
    hash: Option<Hash>
}

impl< const SIZE: usize, Hash, Key, Element> From<Branch< Self>> for Node<SIZE, Hash, Key, Element>
where   Hash: Copy + Clone + PartialEq + std::fmt::Display + crate::hash::traits::Hash + crate::hash::traits::Hashable + Default,
        Key: PartialEq + PartialOrd + Ord + Clone + crate::hash::traits::Hashable ,
        Element: Clone + crate::hash::traits::Hashable 
{
    fn from(branch: Branch< Self>) -> Self {
        Self {
            node_type: NodeType::Branch(branch),
            hash: None
        }
    }
}

impl< const SIZE: usize, Hash, Key, Element> From<Leaf< Self>> for Node< SIZE, Hash, Key, Element>
where   Hash: Copy + Clone + PartialEq + std::fmt::Display + crate::hash::traits::Hash + crate::hash::traits::Hashable + Default,
        Key: PartialEq + PartialOrd + Ord + Clone + crate::hash::traits::Hashable ,
        Element: Clone + crate::hash::traits::Hashable 
{
    fn from(leaf: Leaf< Self>) -> Self {
        Self {
            node_type: NodeType::Leaf(leaf),
            hash: None
        }
    }
}

impl< const SIZE: usize, Hash, Key, Element> PartialEq<Hash> for Node<SIZE, Hash, Key, Element>
where   Hash: Copy + Clone + PartialEq + std::fmt::Display + crate::hash::traits::Hash + crate::hash::traits::Hashable + Default,
        Key: PartialEq + PartialOrd + Ord + Clone + crate::hash::traits::Hashable ,
        Element: Clone + crate::hash::traits::Hashable 
{
    fn eq(&self, other: &Hash) -> bool {
        self.hash.eq(&Some(other.clone()))
    }
}

impl<const SIZE: usize, Hash, Key, Element> self::traits::Node for Node< SIZE, Hash, Key, Element>
where   Hash: Copy + Clone + PartialEq + std::fmt::Display + Default + crate::hash::traits::Hash + crate::hash::traits::Hashable,
        Key: PartialEq + PartialOrd + Ord + Clone + crate::hash::traits::Hashable ,
        Element: Clone + crate::hash::traits::Hashable 
{
    const SIZE: usize = SIZE;

    type Key        = Key;
    type Hash       = Hash;
    type Element    = Element;

    type Leaf       = Leaf< Self>;
    type Branch     = Branch< Self>;

    fn r#as(&self) -> &NodeType<Self::Branch, Self::Leaf> {
        &self.node_type
    }

    fn as_mut(&mut self) -> &mut NodeType<Self::Branch, Self::Leaf> {
       &mut self.node_type
    }

    fn children(&self) -> Vec<&WeakNode< Self>> {
        match self.r#as() {
            NodeType::Branch(branch) => branch.children(),
            _ => vec![]
        }
    }

    fn compute_hash<Nodes: super::nodes::traits::Nodes< Node=Self>>(&self, nodes: &Nodes) -> TreeResult<Self::Hash, Self> {
        Ok(match self.r#as() {
            NodeType::Branch(branch) => branch.compute_hash(nodes)?,
            NodeType::Leaf(leaf) => leaf.compute_hash()
        })
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

    fn is_full(&self) -> bool {
        match self.r#as() {
            NodeType::Branch(branch) => branch.is_full(),
            NodeType::Leaf(leaf) => leaf.is_full()
        }
    }

    fn set_hash(&mut self, hash: Self::Hash) {
        self.hash = Some(hash)
    }
}