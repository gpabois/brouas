use std::marker::PhantomData;

use bumpalo::Bump;
use crate::tree::node::traits::Node as TNode;
use crate::storage::traits::Storage as TStorage;
use self::traits::Nodes as TNodes;

use super::error::TreeError;
use super::node_ref::WeakNode;
use super::result::TreeResult;

pub mod traits {
    use crate::tree::{node::traits::Node as TNode, node_ref::WeakNode, result::TreeResult};
    pub trait Nodes<'a> {
        type Node: TNode<'a>;
        
        fn alloc(&'a self, node: Self::Node) -> WeakNode<'a, Self::Node>;
        fn load_nodes_if_not<WeakNodeIterator: Iterator<Item=&'a WeakNode<'a, Self::Node>>>(&'a self, nodes: WeakNodeIterator) -> TreeResult<(), Self::Node>;
    }
}

pub struct Nodes<'a, Node, Storage>
where Node: TNode<'a>,
      Storage: TStorage<Key=Node::Hash, Value=Node>
{
    ph: PhantomData<&'a ()>,
    arena: Bump,
    store: Storage
}

impl<'a, Node, Storage> From<Storage> for Nodes<'a, Node, Storage>
where Node: TNode<'a>,
      Storage: TStorage<Key=Node::Hash, Value=Node> 
{
    fn from(store: Storage) -> Self {
        Self {ph: Default::default(), arena: Bump::new(), store: store}
    }
}

impl<'a, Node, Storage> Nodes<'a, Node, Storage>
where Node: TNode<'a>,
      Storage: TStorage<Key=Node::Hash, Value=Node> 
{
    fn load_if_not(&'a self, node_ref: &WeakNode<'a, Node>) -> TreeResult<'a, (), Node>
    {
        if node_ref.is_loaded() == false {
            let node_value = self.store.fetch(&node_ref.id())
            .ok_or(
                TreeError::MissingNode(node_ref.id())
            )?;

            let node: &'a mut Node = self.arena.alloc(node_value);

            node_ref.load(node);
        }

        Ok(())
    }
}

impl<'a, Node, Storage> TNodes<'a> for Nodes<'a, Node, Storage>
where Node: TNode<'a>,
      Storage: TStorage<Key=Node::Hash, Value=Node> 
{
    type Node = Node;

    fn load_nodes_if_not<WeakNodeIterator: Iterator<Item=&'a WeakNode<'a, Node>>>(&'a self, nodes: WeakNodeIterator) -> TreeResult<(), Node>
    {
        let result: Result<Vec<_>, _> = nodes.into_iter().map(|node| self.load_if_not(node)).collect();
        result?;
        
        Ok(())
    }


    fn alloc(&'a self, node: Self::Node) -> WeakNode<'a, Self::Node> {
        WeakNode::from(self.arena.alloc(node))
    }


}

