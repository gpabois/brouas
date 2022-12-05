use std::fmt::Display;

use super::error::TreeError;
use super::node::traits::Node as TNode;
use super::node_ref::WeakNode;
use super::nodes::Nodes;
use super::result::{TreeResult};
use super::{alg as tree_alg};
use crate::storage::traits::Storage as TStorage;

#[derive(Default)]
pub struct MutPath<'a, Node>(Vec<&'a mut WeakNode<'a, Node>>)
    where Node: TNode<'a>;

impl<'a, Node> MutPath<'a, Node>
    where Node: TNode<'a>
{
    pub fn new() -> Self {
        Self(vec![])
    }
    
    pub fn last(&self) -> Option<&&'a mut WeakNode<'a, Node>>
    {
        self.0.last()
    }

    pub fn push(&mut self, node_ref: &'a mut WeakNode<'a, Node>)
    {
        self.0.push(node_ref);
    }

    pub fn pop(&mut self) -> Option<&'a mut WeakNode<'a, Node>>
    {
        self.0.pop()
    }
}


#[derive(Default)]
pub struct Path<'a, Node>(Vec<&'a WeakNode<'a, Node>>)
    where Node: TNode<'a>;

impl<'a, Node> Path<'a, Node>
    where Node: TNode<'a>
{
    pub fn new() -> Self {
        Self(vec![])
    }
    
    pub fn last(&self) -> Option<&&'a WeakNode<'a, Node>>
    {
        self.0.last()
    }

    pub fn push(&mut self, node_ref: &'a WeakNode<'a, Node>)
    {
        self.0.push(node_ref);
    }

    pub fn pop(&mut self) -> Option<&'a WeakNode<'a, Node>>
    {
        self.0.pop()
    }
}

#[derive(Default)]
pub struct Tree<'a, Node> where Node: TNode<'a> {
    root: Option<WeakNode<'a, Node>>
}

impl<'a, Node> Display for Tree<'a, Node> 
where Node: TNode<'a>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(root) = &self.root {
            write!(f, "tree::{}", root)
        } else {
            write!(f, "tree::empty")
        }
        
    }
}

impl<'a, Node> std::fmt::Debug for Tree<'a, Node> 
where Node: TNode<'a>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a, Node> Tree<'a, Node>
where Node: TNode<'a>
{
    pub fn empty() -> Self {
        Self {root: None}
    }

    pub fn existing(root: WeakNode<'a, Node>) -> Self {
        Self {root: Some(root)}
    }

    pub fn set_root(&mut self, root: Option<WeakNode<'a, Node>>)
    {
        self.root = root;
    }

    pub fn get_root(&'a self) -> Option<&'a WeakNode<'a, Node>>
    {
        return self.root.as_ref();
    }

    pub fn get_mut_root(&'a mut self) -> Option<&'a mut WeakNode<'a, Node>>
    {
        return self.root.as_mut();
    }
}

pub mod traits {
    use crate::tree::{node::traits::{Node as TNode}, result::TreeResult};

    pub trait TreeTransaction<'a> {
        type Node: TNode<'a>;

        fn insert(&mut self, 
            key: <Self::Node as TNode<'a>>::Key, 
            element: <Self::Node as TNode<'a>>::Element
        ) -> TreeResult<'a, (), Self::Node>;
        
        fn search(&'a self, key: &<Self::Node as TNode<'a>>::Key)
            -> TreeResult<'a, Option<&'a <Self::Node as TNode>::Element>, Self::Node>;
        
        fn search_mut(&'a mut self, key: &<Self::Node as TNode<'a>>::Key)
            -> TreeResult<'a, Option<&'a mut <Self::Node as TNode>::Element>, Self::Node>;
        
    }
}

/// Merkle B+ Tree
pub struct TreeTransaction<'a, Node, Storage> 
where Node: TNode<'a>, 
      Storage: TStorage<Key=Node::Hash, Value=Node>
{
    tree: Tree<'a, Node>,
    nodes: Nodes<'a, Node, Storage>
}

impl<'a, Node, Storage> TreeTransaction<'a, Node, Storage> 
where Node: TNode<'a>, Storage: crate::storage::traits::Storage<Key=Node::Hash, Value=Node>
{
    pub fn new(tree: Tree<'a, Node>, storage: Storage) -> Self
    {
        Self {
            tree: tree,
            nodes: Nodes::from(storage)
        }
    }
}

impl<'a, Node, Storage> self::traits::TreeTransaction<'a> for TreeTransaction<'a, Node, Storage>
where Node: TNode<'a>, Storage: crate::storage::traits::Storage<Key=Node::Hash, Value=Node>
{
    type Node = Node;
    
    /// Insert an element.
    fn insert(&mut self, key: Node::Key, element: Node::Element)  -> TreeResult<'a, (), Node>
    {
        tree_alg::insert(&mut self.tree, &self.nodes, key, element)
    }

    /// Search an element and returns an immutable reference to it.
    fn search(&'a self, key: &Node::Key) -> TreeResult<'a, Option<&'a Node::Element>, Node>
    {
        tree_alg::search(&self.tree, &self.nodes, key)
    }

    /// Search an element and returns an mutable reference to it.
    fn search_mut(&'a mut self, key: &Node::Key) -> TreeResult<'a, Option<&'a mut Node::Element>, Node>
    {
        tree_alg::search_mut(&mut self.tree, &self.nodes, key)
    }

}

#[cfg(test)]
mod tests {
    use super::{*, traits::TreeTransaction as TTreeTransaction};
    use crate::{storage::{InMemory, MutRefStorage}, tree::Node, hash::Sha256};

    #[derive(Clone)]
    pub struct TestElement {
        data: u8
    }
    
    impl crate::hash::traits::Hashable for TestElement
    {
        fn hash<H: crate::hash::traits::Hasher>(&self, hasher: &mut H) {
            hasher.update([self.data]);
        }
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
    pub struct TestKey {
        data: u8
    }

    impl TestKey {
        pub fn new(data: u8) -> Self {
            Self {
                data: data
            }
        }
    }

    impl crate::hash::traits::Hashable for TestKey {
        fn hash<H: crate::hash::traits::Hasher>(&self, hasher: &mut H) {
            hasher.update([self.data])
        }
    }


}