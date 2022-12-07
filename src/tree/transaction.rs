use super::node::traits::Node as TNode;
use super::nodes::Nodes;
use super::result::{TreeResult};
use super::{alg as tree_alg, Tree};
use crate::storage::traits::Storage as TStorage;

pub mod traits {
    use crate::tree::{node::traits::{Node as TNode}, result::TreeResult};
    
    pub trait TreeTransaction {
        type Node: TNode;

        fn insert(&mut self, 
            key: <Self::Node as TNode>::Key, 
            element: <Self::Node as TNode>::Element
        ) -> TreeResult< (), Self::Node>;
        
        fn search(& self, key: &<Self::Node as TNode>::Key)
            -> TreeResult< Option<& <Self::Node as TNode>::Element>, Self::Node>;
        
        fn search_mut(& mut self, key: &<Self::Node as TNode>::Key)
            -> TreeResult< Option<& mut <Self::Node as TNode>::Element>, Self::Node>;
        
        fn commit(&mut self) -> TreeResult<Option<<Self::Node as TNode>::Hash>, Self::Node>;

    }
}

/// Merkle B+ Tree
pub struct TreeTransaction< Node, Storage> 
where Node: TNode, 
      Storage: TStorage<Key=Node::Hash, Value=Node>
{
    tree: Tree<Node>,
    nodes: Nodes<Node, Storage>
}

impl< Node, Storage> TreeTransaction< Node, Storage> 
where Node: TNode, Storage: crate::storage::traits::Storage<Key=Node::Hash, Value=Node>
{
    pub fn new(tree: Tree<Node>, storage: Storage) -> Self
    {
        Self {
            tree: tree,
            nodes: Nodes::from(storage)
        }
    }
}

impl< Node, Storage> self::traits::TreeTransaction for TreeTransaction< Node, Storage>
where Node: TNode + 'static, Storage: crate::storage::traits::Storage<Key=Node::Hash, Value=Node>
{
    type Node = Node;
    
    /// Insert an element.
    fn insert(&mut self, key: Node::Key, element: Node::Element)  -> TreeResult< (), Node>
    {
        tree_alg::insert(&mut self.tree, &mut self.nodes, key, element)
    }

    /// Search an element and returns an immutable reference to it.
    fn search(& self, key: &Node::Key) -> TreeResult< Option<& Node::Element>, Node>
    {
        tree_alg::search(&self.tree, &self.nodes, key)
    }

    /// Search an element and returns an mutable reference to it.
    fn search_mut(& mut self, key: &Node::Key) -> TreeResult< Option<& mut Node::Element>, Node>
    {
        tree_alg::search_mut(&mut self.tree, &mut self.nodes, key)
    }

    fn commit(&mut self) -> TreeResult<Option<<Self::Node as TNode>::Hash>, Self::Node> 
    {
        let root_hash = tree_alg::calculate_hashes(&mut self.tree, &mut self.nodes)?;
        Ok(root_hash)
    }

}

#[cfg(test)]
mod tests {
    use crate::{storage::{InMemory, MutRefStorage}, tree::{Node, new_merkle_bp_tree}, hash::Sha256};

    use super::TreeTransaction;

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
    pub fn test_insert()
    {
        let tree = new_merkle_bp_tree::<10, Sha256, TestKey, TestElement>();
        let tx = TreeTransaction::new(tree);
    }

}