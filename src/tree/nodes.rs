use bumpalo::Bump;
use crate::tree::node::traits::Node as TNode;
use crate::storage::traits::Storage as TStorage;

use super::error::TreeError;
use super::node_ref::NodeRef;
use super::result::TreeResult;

pub mod traits {
    use crate::tree::{node::traits::Node as TNode, node_ref::NodeRef, result::TreeResult};
    pub trait Nodes {
        type Node: TNode;
        
        fn alloc<'a>(&'a self, node: Self::Node) -> NodeRef<'a, Self::Node>;
        fn load_nodes_if_not<'a, NodeRefIterator: Iterator<Item=&'a NodeRef<'a, Self::Node>>>(&'a self, nodes: NodeRefIterator) -> TreeResult<(), Self::Node>;
    }
}

pub struct Nodes<Node, Storage>
where Node: TNode,
      Storage: TStorage<Key=Node::Hash, Value=Node>
{
    arena: Bump,
    store: Storage
}

impl<Node, Storage> Nodes<Node, Storage>
where Node: TNode,
      Storage: TStorage<Key=Node::Hash, Value=Node> 
{
    pub fn load_nodes_if_not<'a, NodeRefIterator: Iterator<Item=&'a NodeRef<'a, Node>>>(&'a self, nodes: NodeRefIterator) -> TreeResult<(), Node>
    {
        let result: Result<Vec<_>, _> = nodes.into_iter().map(self.load_if_not).collect();
        result?;
        
        Ok(())
    }

    pub fn load_if_not(&self, node_ref: &NodeRef<Node>) -> TreeResult<(), Node>
    {
        if node_ref.is_loaded() == false {
            let node_value = self.store.fetch(&node_ref.id())
            .ok_or(
                TreeError::MissingNode(node_ref.id())
            )?;

            let node = self.arena.alloc(node_value);

            node_ref.load(node);
        }

        Ok(())
    }
}

