use std::fmt::Display;

use super::error::TreeError;
use super::node::traits::Node as TNode;
use super::node_ref::NodeRef;
use super::nodes::Nodes;
use super::result::{TreeResult};
use super::{alg as tree_alg};

#[derive(Default)]
pub struct Path<'a, Node>(Vec<&'a NodeRef<'a, Node>>)
    where Node: TNode;

impl<'a, Node> Path<'a, Node>
    where Node: TNode
{
    pub fn new() -> Self {
        Self(vec![])
    }
    
    pub fn last(&self) -> Option<&&NodeRef<'a, Node>>
    {
        self.0.last()
    }

    pub fn push(&mut self, node_ref: &NodeRef<'a, Node>)
    {
        self.0.push(node_ref);
    }

    pub fn pop(&mut self) -> Option<&NodeRef<'a, Node>>
    {
        self.0.pop()
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct Tree<'a, Node> where Node: TNode {
    root: Option<NodeRef<'a, Node>>
}

impl<'a, Node> Display for Tree<'a, Node> 
where Node: TNode
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
where Node: TNode
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a, Node> Tree<'a, Node>
where Node: TNode
{
    pub fn empty() -> Self {
        Self {root: None}
    }

    pub fn existing(root: NodeRef<'a, Node>) -> Self {
        Self {root: Some(root)}
    }

    pub fn set_root(&mut self, root: Option<NodeRef<'a, Node>>)
    {
        self.root = root;
    }

    pub fn get_root(&self) -> Option<&NodeRef<'a, Node>>
    {
        return self.root.as_ref();
    }
}

pub mod traits {
    use crate::tree::{node::traits::{Node as TNode}, error::TreeError, result::TreeResult};

    pub trait TreeTransaction {
        type Node: TNode;

        fn insert(&mut self, 
            key: <Self::Node as TNode>::Key, 
            element: <Self::Node as TNode>::Element
        ) -> TreeResult<(), Self::Node>;
        
        fn search<'a>(&'a self, key: &<Self::Node as TNode>::Key)
            -> TreeResult<Option<&'a <Self::Node as TNode>::Element>, Self::Node>;
        
        fn search_mut<'a>(&'a mut self, key: &<Self::Node as TNode>::Key)
            -> TreeResult<Option<&'a mut <Self::Node as TNode>::Element>, Self::Node>;
        
    }
}

/// Merkle B+ Tree
pub struct TreeTransaction<'a, Node, Storage> 
where Node: TNode, Storage: crate::storage::traits::Storage<Key=Node::Hash, Value=Node>
{
    tree: Tree<'a, Node>,
    nodes: Nodes<Node, Storage>
}

impl<'a, Node, Storage> TreeTransaction<'a, Node, Storage> 
where Node: TNode, Storage: crate::storage::traits::Storage<Key=Node::Hash, Value=Node>
{
    pub fn new(tree: Tree<'a, Node>, storage: Storage) -> Self
    {
        Self {
            tree: tree,
            nodes: Nodes::from(storage)
        }
    }
}

impl<'a, Node, Storage> self::traits::TreeTransaction for TreeTransaction<'a, Node, Storage>
where Node: TNode, Storage: crate::storage::traits::Storage<Key=Node::Hash, Value=Node>
{
    type Node = Node;
    
    /// Insert an element.
    fn insert(&mut self, key: Node::Key, element: Node::Element)  -> Result<(), TreeError<Node>>
    {
        tree_alg::insert(&mut self.tree_ref, &mut self.nodes, key, element)
    }

    /// Search an element and returns an immutable reference to it.
    fn search<'b>(&'b self, key: &Node::Key) -> TreeResult<Option<&'b Node::Element>, Node>
    {
        tree_alg::search(&self.tree_ref, &self.nodes, key)
    }

    /// Search an element and returns an mutable reference to it.
    fn search_mut<'b>(&'b mut self, key: &Node::Key) -> TreeResult<Option<&'b mut Node::Element>, Node>
    {
        tree_alg::search_mut(&self.tree, &self.nodes, key)
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

    #[test]
    fn test_tree_insert_and_search() -> Result<(), TreeError<Node<'static, 3, Sha256, TestKey, TestElement>>>
    {
        let mut storage = InMemory::<Sha256, Node<3, Sha256, TestKey, TestElement>>::new();
        let tree;
        
        // Create a transaction to insert our element
        {
            let mut transaction = TreeTransaction::new(
                Tree::empty(),  
                MutRefStorage::from(&mut storage)
            );
            transaction.insert(TestKey::new(0), TestElement{data: 0x10})?;
            tree = transaction.commit()?;

            assert_ne!(tree, Tree::empty());
        }

        // Create a transaction to search our element
        {
            let transaction = TreeTransaction::new(
                tree,  
                MutRefStorage::from(&mut storage)
            );    
            let element = transaction.search(&TestKey::new(0))?.expect("Expecting an element");
            assert_eq!(element.data, 0x10);
        }

        Ok(())
    }
}