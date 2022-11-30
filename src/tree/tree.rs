use std::fmt::Display;

use super::NodeRef;
use super::Nodes;
use super::error::TreeError;
use super::node::traits::Node as TNode;
use super::{alg as tree_alg};

#[derive(Default)]
pub struct Path<Hash: Clone + PartialEq>(Vec<NodeRef<Hash>>);

impl<Hash: Clone+PartialEq> Path<Hash>
{
    pub fn new() -> Self {
        Self(vec![])
    }
    
    pub fn last<'a>(&'a self) -> Option<&'a NodeRef<Hash>>
    {
        self.0.last()
    }

    pub fn push(&mut self, node_ref: NodeRef<Hash>)
    {
        self.0.push(node_ref);
    }

    pub fn pop(&mut self) -> Option<NodeRef<Hash>>
    {
        self.0.pop()
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct Tree<Hash: Clone + PartialEq> {
    root: Option<NodeRef<Hash>>
}

impl<Hash: Clone + PartialEq + Display> Display for Tree<Hash> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(root) = &self.root {
            write!(f, "tree::{}", root)
        } else {
            write!(f, "tree::empty")
        }
        
    }
}

impl<Hash: Clone + PartialEq + Display> std::fmt::Debug for Tree<Hash> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<Hash: Clone + PartialEq> Tree<Hash>
{
    pub fn empty() -> Self {
        Self {root: None}
    }

    pub fn existing(root: NodeRef<Hash>) -> Self {
        Self {root: Some(root)}
    }

    pub fn set_root(&mut self, root: Option<NodeRef<Hash>>)
    {
        self.root = root;
    }

    pub fn get_root<'a>(&'a self) -> Option<&'a NodeRef<Hash>>
    {
        return self.root.as_ref();
    }
}

pub mod traits {
    use crate::tree::{node::traits::{Node, Nodes}, error::TreeError};

    use super::Tree;

    pub trait TreeTransaction {
        type Nodes: crate::tree::node::traits::Nodes;

        fn insert(&mut self, 
            key: <<Self::Nodes as Nodes>::Node as Node>::Key, 
            element: <<Self::Nodes as Nodes>::Node as Node>::Element
        ) -> Result<(), TreeError<<<Self::Nodes as Nodes>::Node as Node>::Hash>>;
        fn search<'a>(&'a self, key: &<<Self::Nodes as Nodes>::Node as Node>::Key)
            -> Result<Option<&'a <<Self::Nodes as Nodes>::Node as Node>::Element>, TreeError<<<Self::Nodes as Nodes>::Node as Node>::Hash>>;
        
        fn search_mut<'a>(&'a mut self, key: &<<Self::Nodes as Nodes>::Node as Node>::Key)
            -> Result<Option<&'a mut <<Self::Nodes as Nodes>::Node as Node>::Element>, TreeError<<<Self::Nodes as Nodes>::Node as Node>::Hash>>;
        
        /// Commit the modifications
        fn commit(&mut self) -> Result<Tree<<<Self::Nodes as Nodes>::Node as Node>::Hash>, TreeError<<<Self::Nodes as Nodes>::Node as Node>::Hash>>;
    }
}

/// Merkle B+ Tree
pub struct TreeTransaction<Nodes> 
where Nodes: crate::tree::node::traits::Nodes
{
    tree_ref: Tree<<Nodes::Node as TNode>::Hash>,
    nodes: Nodes
}

impl<'a, Node, Storage> TreeTransaction<Nodes<Node, Storage>> 
where Node: TNode + 'static,
      Storage: crate::storage::traits::Storage<Key=Node::Hash, Value=Node>
{
    pub fn new(tree_ref: Tree<Node::Hash>, storage: Storage) -> Self
    {
        Self {
            tree_ref: tree_ref,
            nodes: Nodes::from(storage)
        }
    }
}

impl<'a, Nodes> self::traits::TreeTransaction for TreeTransaction<Nodes>
where Nodes: crate::tree::node::traits::Nodes
{
    type Nodes = Nodes;
    
    /// Insert an element.
    fn insert(&mut self, key: <Nodes::Node as TNode>::Key, element: <Nodes::Node as TNode>::Element)  -> Result<(), TreeError<<Nodes::Node as TNode>::Hash>>
    {
        tree_alg::insert(&mut self.tree_ref, &mut self.nodes, key, element)
    }

    /// Search an element and returns an immutable reference to it.
    fn search<'b>(&'b self, key: &<Nodes::Node as TNode>::Key) 
        -> Result<
            Option<&'b <Nodes::Node as TNode>::Element>, 
            TreeError<<Nodes::Node as TNode>::Hash>
        >
    {
        tree_alg::search(&self.tree_ref, &self.nodes, key)
    }

    /// Search an element and returns an mutable reference to it.
    fn search_mut<'b>(&'b mut self, key: &<Nodes::Node as TNode>::Key) -> Result<
        Option<&'b mut <Nodes::Node as TNode>::Element>, 
        TreeError<<Nodes::Node as TNode>::Hash>
        >
    {
        tree_alg::search_mut(&self.tree_ref, &mut self.nodes, key)
    }

    fn commit(&mut self) -> Result<Tree<<Nodes::Node as TNode>::Hash>, TreeError<<Nodes::Node as TNode>::Hash>>{
        let updated_nodes = tree_alg::commit(&mut self.tree_ref, &mut self.nodes);
        self.nodes.save_nodes(updated_nodes);
        Ok(self.tree_ref.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Display;

    use super::{*, traits::TreeTransaction as TTreeTransaction};
    use crate::{storage::{InMemory, MutRefStorage}, tree::Node};

    #[derive(Clone)]
    pub struct TestElement {
        data: u8
    }

    #[derive(Hash, Clone, Eq, PartialEq, PartialOrd)]
    pub struct Sha256
    {

    }

    impl Display for Sha256 {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            todo!()
        }
    }

    #[test]
    fn test_tree_insert_and_search() -> Result<(), TreeError<Sha256>>
    {
        let mut storage = InMemory::<Sha256, Node<3, Sha256, u8, TestElement>>::new();
        let tree;
        
        // Create a transaction to insert our element
        {
            let mut transaction = TreeTransaction::new(
                Tree::empty(),  
                MutRefStorage::from(&mut storage)
            );
            transaction.insert(0, TestElement{data: 0x10})?;
            tree = transaction.commit()?;

            assert_ne!(tree, Tree::empty());
        }

        // Create a transaction to search our element
        {
            let transaction = TreeTransaction::new(
                tree,  
                MutRefStorage::from(&mut storage)
            );    
            let element = transaction.search(&0)?.expect("Expecting an element");
            assert_eq!(element.data, 0x10);
        }

        Ok(())
    }
}