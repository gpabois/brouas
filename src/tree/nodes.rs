use std::marker::PhantomData;

use bumpalo::Bump;
use crate::arena::{Arena, ArenaId};
use crate::tree::node::traits::Node as TNode;
use crate::storage::traits::Storage as TStorage;
use self::traits::Nodes as TNodes;

use super::error::TreeError;
use super::node_ref::{WeakNode, RefNode, RefMutNode};
use super::result::TreeResult;

pub mod traits {
    use crate::tree::{node::traits::Node as TNode, node_ref::{WeakNode, RefNode, RefMutNode}, result::TreeResult};
    pub trait Nodes<'a> {
        type Node: TNode<'a>;
        
        fn alloc(&'a self, node: Self::Node) -> WeakNode<'a, Self::Node>;
        fn upgrade(&'a self, weak: &WeakNode<'a, Self::Node>) -> TreeResult<'a, RefNode<'a, Self::Node>, Self::Node>;
        fn upgrade_mut(&'a mut self, weak: &WeakNode<'a, Self::Node>) -> TreeResult<'a, RefMutNode<'a, Self::Node>, Self::Node>;
        fn load_nodes_if_not<WeakNodeIterator: Iterator<Item=&'a WeakNode<'a, Self::Node>>>(&'a self, nodes: WeakNodeIterator) -> TreeResult<(), Self::Node>;
        fn load_if_not(&'a self, node_ref: &WeakNode<'a, Self::Node>) -> TreeResult<'a, (), Self::Node>;
    }
}

pub struct Nodes<'a, Node, Storage>
where Node: TNode<'a>,
      Storage: TStorage<Key=Node::Hash, Value=Node>
{
    ph: PhantomData<&'a ()>,
    arena: Arena<Node>,
    store: Storage
}

impl<'a, Node, Storage> From<Storage> for Nodes<'a, Node, Storage>
where Node: TNode<'a>,
      Storage: TStorage<Key=Node::Hash, Value=Node> 
{
    fn from(store: Storage) -> Self {
        Self {ph: Default::default(), arena: Arena::new(), store: store}
    }
}

impl<'a, Node, Storage> Nodes<'a, Node, Storage>
where Node: TNode<'a>,
      Storage: TStorage<Key=Node::Hash, Value=Node> 
{
    fn load_if_not(&'a self, node_ref: &WeakNode<'a, Node>) -> TreeResult<'a, (), Node>
    {
        if node_ref.is_loaded() == false {
            let node_value = self.store.fetch(&node_ref.as_node_id().unwrap())
            .ok_or(
                TreeError::MissingNode(node_ref.as_node_id().unwrap().clone())
            )?;

            let weak_node = self.arena.alloc(node_value);

            node_ref.load(weak_node);
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

    fn load_if_not(&'a self, node_ref: &WeakNode<'a, Node>) -> TreeResult<'a, (), Node>
    {
        if node_ref.is_loaded() == false {
            let node_value = self.store.fetch(&node_ref.as_node_id().unwrap())
            .ok_or(
                TreeError::MissingNode(node_ref.as_node_id().unwrap().clone())
            )?;

            let arena_id: ArenaId = self.arena.alloc(node_value);

            node_ref.load(arena_id);
        }

        Ok(())
    }

    fn alloc(&'a self, node: Self::Node) -> WeakNode<'a, Self::Node> {
        WeakNode::from(self.arena.alloc(node))
    }

    fn upgrade(&'a self, weak: &WeakNode<'a, Self::Node>) -> TreeResult<'a, super::node_ref::RefNode<'a, Self::Node>, Self::Node> {
        self.load_if_not(weak)?;
        let ref_node = self.arena.upgrade(weak.as_arena_id().unwrap()).unwrap();
        Ok(RefNode::from(ref_node))
    }

    fn upgrade_mut(&'a mut self, weak: &WeakNode<'a, Self::Node>) -> TreeResult<'a, super::node_ref::RefMutNode<'a, Self::Node>, Self::Node> {
        self.load_if_not(weak)?;
        let ref_mut_node = self.arena.upgrade_mut(weak.as_arena_id().unwrap()).unwrap();
        Ok(RefMutNode::from(ref_mut_node))
    }



}

