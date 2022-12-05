use std::{cell::{UnsafeCell, RefCell, Ref}, ops::{Deref, DerefMut}, borrow::BorrowMut};
use crate::tree::node::traits::Node as TNode;
use std::borrow::Borrow;

use super::{nodes::traits::Nodes as TNodes, result::TreeResult, error::TreeError};

pub enum CoreWeakNode<'a, Node>
where Node: TNode<'a>
{
    Node(&'a mut Node),
    Id(Node::Hash)
}

impl<'a, Node> From<&'a mut Node> for CoreWeakNode<'a, Node>
where Node: TNode<'a>
{
    fn from(node: &'a mut Node) -> Self {
        Self::Node(node)
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
pub struct RefNode<'a, Node>(&'a Node);
impl<'a, Node> RefNode<'a, Node>
{
    pub fn take(self) -> &'a Node {
        self.0
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

impl<'a, Node> From<&'a mut Node> for WeakNode<'a, Node>
where Node: TNode<'a>
{
    fn from(node: &'a mut Node) -> Self {
        Self(UnsafeCell::new(CoreWeakNode::from(node)))
    }
}

impl<'a, Node> WeakNode<'a, Node>
where Node: TNode<'a>
{
    pub fn upgrade_mut<Nodes: TNodes<'a, Node=Node>>(&'a mut self, nodes: &Nodes) -> TreeResult<'a, RefMutNode<'a, Node>, Node>
    {
        if let CoreWeakNode::Node(node_ref) = self.0.get_mut() {
            return Ok(RefMutNode(node_ref));
        } else {
            //nodes.load_nodes_if_not([self].into_iter())?;
            let mut_node_ref = self.mut_node().unwrap();
            return Ok(RefMutNode(mut_node_ref))
        }
    }

    pub fn upgrade<Nodes: TNodes<'a, Node=Node>>(&'a self, nodes: &Nodes) -> TreeResult<RefNode<'a, Node>, Node>
    {
        if let CoreWeakNode::Node(node_ref) = self.0.get_mut() {
            return Ok(RefNode(node_ref));
        } else {
            //nodes.load_nodes_if_not([self].into_iter())?;
            let node_ref = self.node().unwrap();
            return Ok(RefNode(node_ref))
        }
    }

    fn mut_node(&'a self) -> Option<&'a mut Node>
    {
        if let CoreWeakNode::Node(node_ref) = self.0.get_mut() {
            Some(node_ref)
        } else {
            None
        }
    }

    fn node(&'a self) -> Option<&'a mut Node>
    {
        if let CoreWeakNode::Node(node_ref) = self.0.get_mut() {
            Some(node_ref)
        } else {
            None
        }
    }

    pub fn is_loaded(&self) -> bool {
        if let CoreWeakNode::Node(_) = self.0.get_mut() {
            true
        } else {
            false
        }
    }

    pub fn id(&self) -> Node::Hash {
        unsafe {
            match self.0.get().as_ref().unwrap() {
                CoreWeakNode::Id(id) => id.clone(),
                CoreWeakNode::Node(node) => {
                    node.compute_hash()
                }
            }
        }
    }

    pub fn load(&self, node: &'a mut Node) {
        unsafe {
            *self.0.get().as_mut().unwrap() = CoreWeakNode::Node(node);
        }
    }

    pub fn to_owned(&self) -> Self {
        let base = self.0.into_inner();
        Self(UnsafeCell::new(base))
    }
}