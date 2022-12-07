use crate::arena::{Arena, ArenaId};
use crate::tree::node::traits::Node as TNode;
use crate::storage::traits::Storage as TStorage;
use self::traits::Nodes as TNodes;

use super::error::TreeError;
use super::node_ref::{WeakNode, RefNode, RefMutNode};
use super::result::TreeResult;

pub mod traits {
    use crate::tree::{node::traits::Node as TNode, node_ref::{WeakNode, RefNode, RefMutNode}, result::TreeResult};
    pub trait Nodes {
        type Node: TNode + 'static;
        
        fn alloc(& self, node: Self::Node) -> WeakNode< Self::Node>;
        fn upgrade(& self, weak: &WeakNode< Self::Node>) -> TreeResult< RefNode<Self::Node>, Self::Node>;
        fn upgrade_mut(& mut self, weak: &WeakNode< Self::Node>) -> TreeResult<RefMutNode<Self::Node>, Self::Node>;
        fn load_nodes_if_not<'a, WeakNodeIterator: Iterator<Item=&'a WeakNode<Self::Node>>>(&self, nodes: WeakNodeIterator) -> TreeResult<(), Self::Node>;
        fn load_if_not(& self, node_ref: &WeakNode< Self::Node>) -> TreeResult< (), Self::Node>;
    }
}

pub struct Nodes< Node, Storage>
where Node: TNode,
      Storage: TStorage<Key=Node::Hash, Value=Node>
{
    arena: Arena<Node>,
    store: Storage
}

impl< Node, Storage> From<Storage> for Nodes< Node, Storage>
where Node: TNode,
      Storage: TStorage<Key=Node::Hash, Value=Node> 
{
    fn from(store: Storage) -> Self {
        Self {arena: Arena::new(), store: store}
    }
}

impl< Node, Storage> Nodes< Node, Storage>
where Node: TNode,
      Storage: TStorage<Key=Node::Hash, Value=Node> 
{
    fn load_if_not(&self, node_ref: &WeakNode< Node>) -> TreeResult< (), Node>
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

impl< Node, Storage> TNodes for Nodes< Node, Storage>
where Node: TNode + 'static,
      Storage: TStorage<Key=Node::Hash, Value=Node> 
{
    type Node = Node;

    fn load_nodes_if_not<'a, WeakNodeIterator: Iterator<Item=&'a WeakNode<Node>>>(& self, nodes: WeakNodeIterator) -> TreeResult<(), Node>
    {
        let result: Result<Vec<_>, _> = nodes.into_iter().map(|node| self.load_if_not(node)).collect();
        result?;
        
        Ok(())
    }

    fn load_if_not(& self, node_ref: &WeakNode< Node>) -> TreeResult<(), Node>
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

    fn alloc(& self, node: Self::Node) -> WeakNode< Self::Node> {
        WeakNode::from(self.arena.alloc(node))
    }

    fn upgrade(& self, weak: &WeakNode< Self::Node>) -> TreeResult<RefNode<Self::Node>, Self::Node> {
        self.load_if_not(weak)?;
        let ref_node = self.arena.upgrade(&weak.as_arena_id().unwrap()).unwrap();
        Ok(RefNode::from(ref_node))
    }

    fn upgrade_mut(& mut self, weak: &WeakNode< Self::Node>) -> TreeResult<RefMutNode<Self::Node>, Self::Node> {
        self.load_if_not(weak)?;
        let ref_mut_node = self.arena.upgrade_mut(&weak.as_arena_id().unwrap()).unwrap();
        Ok(RefMutNode::from(ref_mut_node))
    }



}

