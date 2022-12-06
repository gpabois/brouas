use std::{cell::{UnsafeCell}};
use crate::{tree::node::traits::Node as TNode, arena::ArenaId};

use super::{nodes::traits::Nodes as TNodes, result::TreeResult};

pub enum CoreWeakNode<'a, Node>
where Node: TNode<'a>
{
    ArenaId(ArenaId),
    Id(Node::Hash)
}

impl<'a, Node> From<ArenaId> for CoreWeakNode<'a, Node>
where Node: TNode<'a>
{
    fn from(arena_id: ArenaId) -> Self {
        Self::ArenaId(arena_id)
    }
}

impl<'a, Node> Default for CoreWeakNode<'a, Node>
where Node: TNode<'a>
{
    fn default() -> Self {
        Self::Id(Node::Hash::default())
    }
}

pub struct RefMutNode<'a, Node>(&'a mut Node);
impl<'a, Node> RefMutNode<'a, Node>
{
    pub fn take(self) -> &'a mut Node {
        self.0
    }
}

impl<'a, Node> From<&'a mut Node> for RefMutNode<'a, Node>
{
    fn from(node: &'a mut Node) -> Self {
        Self(node)
    }
}

pub struct RefNode<'a, Node>(&'a Node);
impl<'a, Node> RefNode<'a, Node>
{
    pub fn take(self) -> &'a Node {
        self.0
    }
}

impl<'a, Node> From<&'a Node> for RefNode<'a, Node>
{
    fn from(node: &'a Node) -> Self {
        Self(node)
    }
}

#[derive(Default)]
pub struct WeakNode<'a, Node>(UnsafeCell<CoreWeakNode<'a, Node>>)
where Node: TNode<'a>;

impl<'a, Node> std::fmt::Display for WeakNode<'a, Node>
where Node: TNode<'a>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "node::{}", self.id())
    }
}

impl<'a, Node> From<ArenaId> for WeakNode<'a, Node>
where Node: TNode<'a>
{
    fn from(arena_id: ArenaId) -> Self {
        Self(UnsafeCell::new(CoreWeakNode::from(arena_id)))
    }
}

impl<'a, Node> WeakNode<'a, Node>
where Node: TNode<'a>
{
    pub fn to_owned(&mut self) -> Self {
        Self(UnsafeCell::new(self.0.into_inner()))
    }

    pub fn upgrade_mut<Nodes: TNodes<'a, Node=Node>>(&'a mut self, nodes: &mut Nodes) -> TreeResult<'a, RefMutNode<'a, Node>, Node>
    {
        nodes.upgrade_mut(self)
    }

    pub fn upgrade<Nodes: TNodes<'a, Node=Node>>(&'a self, nodes: &Nodes) -> TreeResult<RefNode<'a, Node>, Node>
    {
        nodes.upgrade(self)
    }

    pub fn is_loaded(&self) -> bool {
        if let CoreWeakNode::ArenaId(_) = self.0.get_mut() {
            true
        } else {
            false
        }
    }

    pub fn load(&self, arena_id: ArenaId) {
        unsafe {
            *self.0.get().as_mut().unwrap() = CoreWeakNode::ArenaId(arena_id);
        }
    }

    pub fn as_arena_id(&self) -> Option<&ArenaId>
    {
        match self.0.get_mut() {
            CoreWeakNode::ArenaId(id) => Some(id),
            _ => None
        }
    }

    pub fn as_node_id(&self) -> Option<&Node::Hash>
    {
        match self.0.get_mut() {
            CoreWeakNode::Id(id) => Some(id),
            _ => None
        }
    }

}