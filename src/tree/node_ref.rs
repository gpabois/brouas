use std::{cell::{UnsafeCell}, ops::{Deref, DerefMut}};
use crate::tree::node::traits::Node as TNode;

pub enum BaseNodeRef<'a, Node>
where Node: crate::tree::node::traits::Node
{
    Node(&'a mut Node),
    Id(Node::Hash)
}

impl<'a, Node> Default for BaseNodeRef<'a, Node>
where Node: crate::tree::node::traits::Node
{
    fn default() -> Self {
        Self::Id(Node::Hash::default())
    }
}

#[derive(Default)]
pub struct NodeRef<'a, Node>(UnsafeCell<BaseNodeRef<'a, Node>>)
where Node: crate::tree::node::traits::Node;

impl<'a, Node> NodeRef<'a, Node>
where Node: TNode
{
    pub fn is_loaded(&self) -> bool {
        if let BaseNodeRef::Node(_) = self.0.borrow() {
            true
        } else {
            false
        }
    }

    pub fn id(&self) -> Node::Hash {
        match self.0.get_mut() {
            BaseNodeRef::Id(id) => id,
            BaseNodeRef::Node(node) => node.compute_hash()
        }
    }

    pub fn load(&self, node: &mut Node) {
        *self.0.get_mut() = NodeRef::Node(node);
    }
}

impl<'a, Node> Deref for NodeRef<'a, Node>
where Node: TNode
{
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        match *self.0.get_mut() {
            BaseNodeRef::Node(node) => node,
            BaseNodeRef::Id(_) => panic!("node has not been loaded.")
        }
    }
}

impl<'a, Node> DerefMut for NodeRef<'a, Node>
where Node: TNode
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match *self.0.get_mut() {
            BaseNodeRef::Node(node) => node,
            BaseNodeRef::Id(_) => panic!("node has not been loaded.")
        }
    }
}