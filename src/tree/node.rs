use crate::arena::{tl_arena::TLArena, tl_arena::traits::TLArena as TTLArena, traits::Arena};

use super::{NodeRef, Leaf, Branch};
use super::leaf::traits::Leaf as TLeaf;
use super::branch::traits::Branch as TBranch;

#[derive(Clone)]
pub enum NodeType<Branch, Leaf>
where Branch: crate::tree::branch::traits::Branch,
      Leaf: crate::tree::leaf::traits::Leaf
{
    Leaf(Leaf),
    Branch(Branch)
}

impl<Branch, Leaf> NodeType<Branch, Leaf>
where Branch: crate::tree::branch::traits::Branch,
      Leaf: crate::tree::leaf::traits::Leaf
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
    use std::collections::VecDeque;

    use crate::arena::tl_element_ref::TLElementRef;
    use crate::tree::error::TreeError;
    use crate::{tree::NodeRef};
    use crate::arena::traits::{Allocator, TLElementRef as TraitTLElementRef};

    use super::NodeType;

    pub trait BorrowNode<Node>
    where Node: self::Node {
        fn borrow_node<'a>(&'a self, node_ref: &NodeRef<Node::Hash>) -> Option<&'a Node>;
    }
    
    pub trait BorrowMutNode<Node>
    where Node: self::Node
    {
        fn borrow_mut_node<'a>(&'a mut self, node_ref: &NodeRef<Node::Hash>) -> Option<&'a mut Node>;
    }
    
    /// Nodes collection trait
    pub trait Nodes: 
        BorrowMutNode<Self::Node> 
        + BorrowNode<Self::Node> 
        + Allocator<TLElementRef<<Self::Node as crate::tree::node::traits::Node>::Hash>, Self::Node>
    {
        type Node: Node + 'static;

        /// Returns the loaded nodes, from bottom to top, starting at the top
        fn get_loaded_nodes(&self, root_ref: NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>) 
        -> Result<VecDeque<NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>>, TreeError<<Self::Node as crate::tree::node::traits::Node>::Hash>>
        {
            let mut loaded_nodes: Vec<(usize, NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>)> = Default::default();
            let mut queue: VecDeque<(usize, NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>)> = Default::default();
            
            queue.push_back((0, root_ref));
            
            while let Some((depth, node_ref)) = queue.pop_front()
            {
                if node_ref.is_loaded()
                {
                    let node = self.borrow_node(&node_ref).ok_or(TreeError::MissingNode(node_ref.clone()))?;
                    
                    let mut children_ref: VecDeque<(usize, NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>)> = node
                    .children()
                    .into_iter()
                    .cloned()
                    .map(|c| (depth + 1, c))
                    .collect();

                    queue.append(&mut children_ref);
                    loaded_nodes.push((depth, node_ref));

                }
            }

            loaded_nodes.sort_unstable_by_key(|(d, _)| *d);
            loaded_nodes.reverse();

            Ok(loaded_nodes.into_iter().map(|(_, n)| n).collect())
        }
        
        /// Persist new or updated nodes
        fn save_nodes(&mut self, nodes: Vec<NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>>);
    }

    /// The MBT Node Trait
    pub trait Node: From<Self::Leaf> + From<Self::Branch> + PartialEq<Self::Hash> + Clone
    {
        const SIZE: usize;

        type Key: Clone + PartialOrd + PartialEq + Ord + crate::hash::traits::Hashable ;
        type Hash: Clone + PartialEq + std::fmt::Display + crate::hash::traits::Hash + crate::hash::traits::Hashable;
        type Element: Clone + crate::hash::traits::Hashable;

        type Leaf: crate::tree::leaf::traits::Leaf<Node=Self>;
        type Branch: crate::tree::branch::traits::Branch<Node=Self>;

        fn r#as(&self) -> &NodeType<Self::Branch, Self::Leaf>;
        fn as_mut(&mut self) -> &mut NodeType<Self::Branch, Self::Leaf>;

        /// Get the children of the node
        fn children<'a>(&'a self) -> Vec<&'a NodeRef<Self::Hash>>;

        /// Compute the hash of the node
        fn compute_hash<Nodes: BorrowNode<Self>>(&self, nodes: &Nodes) -> Self::Hash;
        
        fn set_hash(&mut self, hash: Self::Hash);
        fn get_hash(&self) -> Option<Self::Hash>;

        /// Split the node
        fn split(&mut self) -> (Self::Key, Self);
        
        /// The node is full ?
        fn is_full(&self) -> bool;
    }

}

#[derive(Clone)]
pub struct Node<const SIZE: usize, Hash, Key, Element>
where   Hash: Clone + PartialEq + std::fmt::Display + crate::hash::traits::Hash + crate::hash::traits::Hashable,
        Key: PartialEq + PartialOrd + Ord + Clone + crate::hash::traits::Hashable,
        Element: Clone + crate::hash::traits::Hashable 
{
    node_type: NodeType<Branch<Self>, Leaf<Self>>,
    hash: Option<Hash>
}

impl<const SIZE: usize, Hash, Key, Element> From<Branch<Self>> for Node<SIZE, Hash, Key, Element>
where   Hash: Clone + PartialEq + std::fmt::Display + crate::hash::traits::Hash+ crate::hash::traits::Hashable,
        Key: PartialEq + PartialOrd + Ord + Clone + crate::hash::traits::Hashable ,
        Element: Clone + crate::hash::traits::Hashable 
{
    fn from(branch: Branch<Self>) -> Self {
        Self {
            node_type: NodeType::Branch(branch),
            hash: None
        }
    }
}

impl<const SIZE: usize, Hash, Key, Element> From<Leaf<Self>> for Node<SIZE, Hash, Key, Element>
where   Hash: Clone + PartialEq + std::fmt::Display + crate::hash::traits::Hash + crate::hash::traits::Hashable,
        Key: PartialEq + PartialOrd + Ord + Clone + crate::hash::traits::Hashable ,
        Element: Clone + crate::hash::traits::Hashable 
{
    fn from(leaf: Leaf<Self>) -> Self {
        Self {
            node_type: NodeType::Leaf(leaf),
            hash: None
        }
    }
}

impl<const SIZE: usize, Hash, Key, Element> PartialEq<Hash> for Node<SIZE, Hash, Key, Element>
where   Hash: Clone + PartialEq + std::fmt::Display + crate::hash::traits::Hash + crate::hash::traits::Hashable,
        Key: PartialEq + PartialOrd + Ord + Clone + crate::hash::traits::Hashable ,
        Element: Clone + crate::hash::traits::Hashable 
{
    fn eq(&self, other: &Hash) -> bool {
        self.hash.eq(&Some(other.clone()))
    }
}

impl<const SIZE: usize, Hash, Key, Element> self::traits::Node for Node<SIZE, Hash, Key, Element>
where   Hash: Clone + PartialEq + std::fmt::Display + crate::hash::traits::Hash + crate::hash::traits::Hashable,
        Key: PartialEq + PartialOrd + Ord + Clone + crate::hash::traits::Hashable ,
        Element: Clone + crate::hash::traits::Hashable 
{
    const SIZE: usize = SIZE;

    type Key        = Key;
    type Hash       = Hash;
    type Element    = Element;

    type Leaf       = Leaf<Self>;
    type Branch     = Branch<Self>;

    fn r#as(&self) -> &NodeType<Self::Branch, Self::Leaf> {
        &self.node_type
    }

    fn as_mut(&mut self) -> &mut NodeType<Self::Branch, Self::Leaf> {
       &mut self.node_type
    }

    fn children<'a>(&'a self) -> Vec<&'a NodeRef<Self::Hash>> {
        match self.r#as() {
            NodeType::Branch(branch) => branch.children(),
            _ => vec![]
        }
    }

    fn compute_hash<Nodes: traits::BorrowNode<Self>>(&self, nodes: &Nodes) -> Self::Hash {
        match self.r#as() {
            NodeType::Branch(branch) => branch.compute_hash(nodes),
            NodeType::Leaf(leaf) => leaf.compute_hash()
        }
    }

    fn set_hash(&mut self, hash: Self::Hash) {
        self.hash = Some(hash)
    }

    fn get_hash(&self) -> Option<Self::Hash> {
        return self.hash.clone()
    }

    fn split(&mut self) -> (Self::Key, Self) {
        match self.as_mut() {
            NodeType::Branch(branch) => {
                let (key, right_branch) = branch.split_branch();
                let right_node = Self::from(right_branch);
                (key, right_node)
            },
            NodeType::Leaf(leaf) => {
                let (key, right_leaf) = leaf.split_leaf();
                let right_node = Self::from(right_leaf);
                (key, right_node)
            }
        }
    }

    fn is_full(&self) -> bool {
        match self.r#as() {
            NodeType::Branch(branch) => branch.is_full(),
            NodeType::Leaf(leaf) => leaf.is_full()
        }
    }
}

pub struct Nodes<Node, Storage>
where
    Node: crate::tree::node::traits::Node,
    Storage: crate::storage::traits::Storage<Key=Node::Hash, Value=Node>
{
    arena: TLArena<Storage>
}

impl<Node, Storage> Nodes<Node, Storage>
where
    Node: crate::tree::node::traits::Node,
    Storage: crate::storage::traits::Storage<Key=Node::Hash, Value=Node>
{
    pub fn new(storage: Storage) -> Self {
       Self {
            arena: TLArena::new(storage)
       }
    }
}

impl<Node, Storage> From<Storage> for Nodes<Node, Storage>
where
    Node: crate::tree::node::traits::Node,
    Storage: crate::storage::traits::Storage<Key=Node::Hash, Value=Node>
{
    fn from(storage: Storage) -> Self {
       Self::new(storage)
    }
}


impl<Node, Storage>
    crate::arena::traits::Allocator<NodeRef<Node::Hash>, Node> for Nodes<Node, Storage>
where
    Node: crate::tree::node::traits::Node,
    Storage: crate::storage::traits::Storage<Key=Node::Hash, Value=Node>
{
    fn allocate(&mut self, element: Node) -> NodeRef<Node::Hash> {
        self.arena.allocate(element)
    }
}

impl<Node, Storage>
    crate::tree::node::traits::BorrowNode<Node> for Nodes<Node, Storage>
where
    Node: crate::tree::node::traits::Node,
    Storage: crate::storage::traits::Storage<Key=Node::Hash, Value=Node>
{
    fn borrow_node<'a>(&'a self, node_ref: &NodeRef<<Node as traits::Node>::Hash>) -> Option<&'a Node> {
        self.arena.borrow_element(node_ref)
    }
}

impl<Node, Storage>
    crate::tree::node::traits::BorrowMutNode<Node> for Nodes<Node, Storage>
where
    Node: crate::tree::node::traits::Node,
    Storage: crate::storage::traits::Storage<Key=Node::Hash, Value=Node>
{
    fn borrow_mut_node<'a>(&'a mut self, node_ref: &NodeRef<<Node as traits::Node>::Hash>) -> Option<&'a mut Node> {
        self.arena.borrow_mut_element(node_ref)
    }
}

impl<Node, Storage>
    self::traits::Nodes for Nodes<Node, Storage>
where
    Node: crate::tree::node::traits::Node + 'static,
    Storage: crate::storage::traits::Storage<Key=Node::Hash, Value=Node>
{
    type Node = Node;

    fn save_nodes(&mut self, nodes: Vec<super::NodeRef<<Self::Node as crate::tree::node::traits::Node>::Hash>>) 
    {
        self.arena.save_elements(nodes.into_iter());
    }
}
