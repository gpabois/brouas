use super::node::traits::Node as TNode;
use super::nodes::Nodes;
use super::result::{TreeResult};
use super::{alg as tree_alg, Tree};
use crate::storage::traits::ReadOnlyStorage as TReadOnlyStorage;
use self::traits::TreeTransaction as TraitTreeTransaction;
use self::traits::ReadOnlyTreeTransaction as TraitReadOnlyTransaction;
use crate::storage::traits::Storage as TStorage;

pub mod traits {
    use crate::tree::{node::traits::{Node as TNode}, result::TreeResult};
    
    pub trait ReadOnlyTreeTransaction {
        type Node: TNode;

        fn search<IntoKey>(& self, key: IntoKey)
            -> TreeResult< Option<&<Self::Node as TNode>::Element>, Self::Node>
        where IntoKey: Into<<Self::Node as TNode>::Key>;
                
    }

    pub trait TreeTransaction : ReadOnlyTreeTransaction {

        fn insert<IntoKey, IntoElement>(&mut self, 
            key: IntoKey, 
            element: IntoElement
        ) -> TreeResult< (), Self::Node>
        where IntoKey: Into<<Self::Node as TNode>::Key>,
              IntoElement: Into<<Self::Node as TNode>::Element>;
        
        fn search_mut(& mut self, key: &<Self::Node as TNode>::Key)
            -> TreeResult< Option<& mut <Self::Node as TNode>::Element>, Self::Node>;

        fn commit(&mut self) -> TreeResult<Option<<Self::Node as TNode>::Hash>, Self::Node>;

    }
}

/// Merkle B+ Tree
pub struct TreeTransaction< Node, Storage> 
where Node: TNode, 
      Storage: TReadOnlyStorage<Key=Node::Hash, Value=Node>
{
    tree: Tree<Node>,
    nodes: Nodes<Node, Storage>
}

impl< Node, Storage> TreeTransaction< Node, Storage> 
where Node: TNode, Storage: TReadOnlyStorage<Key=Node::Hash, Value=Node>
{
    pub fn new(tree: Tree<Node>, storage: Storage) -> Self
    {
        Self {
            tree: tree,
            nodes: Nodes::from(storage)
        }
    }

}

impl<Node, Storage> TraitReadOnlyTransaction for TreeTransaction<Node, Storage>
where Node: TNode + 'static, Storage: TReadOnlyStorage<Key=Node::Hash, Value=Node>   
{
    type Node = Node;

    fn search<IntoKey>(& self, key: IntoKey)
            -> TreeResult< Option<&<Self::Node as TNode>::Element>, Self::Node>
        where IntoKey: Into<<Self::Node as TNode>::Key> {
        tree_alg::search(&self.tree, &self.nodes, &key.into())
    }
}

impl<Node, Storage> TraitTreeTransaction for TreeTransaction< Node, Storage>
where Node: TNode + 'static, Storage: TReadOnlyStorage<Key=Node::Hash, Value=Node>
{
    fn insert<IntoKey, IntoElement>(&mut self, 
            key: IntoKey, 
            element: IntoElement
        ) -> TreeResult< (), Self::Node>
        where IntoKey: Into<<Self::Node as TNode>::Key>,
              IntoElement: Into<<Self::Node as TNode>::Element> {
        tree_alg::insert(&mut self.tree, &mut self.nodes, key.into(), element.into())
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
    use crate::{storage::{InMemory, ReadOnlyStorage}, tree::{Node, indexes::ByteIndex, Tree}, hash::Sha256};
    use super::TreeTransaction;
    use super::traits::TreeTransaction as TraitTreeTransaction;
    use super::traits::ReadOnlyTreeTransaction as TraitReadOnlyTransaction;

    #[derive(Clone, PartialEq, Debug)]
    pub struct TestElement {
        data: u8
    }
    
    impl From<u8> for TestElement
    {
        fn from(value: u8) -> Self {
            Self{data: value}
        }
    }

    impl crate::hash::traits::Hashable for TestElement
    {
        fn hash<H: crate::hash::traits::Hasher>(&self, hasher: &mut H) {
            hasher.update([self.data]);
        }
    }

    #[test]
    pub fn test_insert()
    {
        let tree = Tree::<Node<10, Sha256, ByteIndex, TestElement>>::empty();
        let storage = InMemory::<Sha256, Node<10, Sha256, ByteIndex, TestElement>>::new();

        let mut tx = TreeTransaction::new(
            tree, 
            ReadOnlyStorage::from(&storage)
        );

        tx.insert( 1, 2).expect("cannot insert element");
        
        assert_eq!(
            *tx.search(1).expect("cannot search element").unwrap(), 
            TestElement::from(2)
        );

        
    }

}