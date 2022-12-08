use std::{cell::{RefCell}, ops::Deref, rc::{Rc, Weak}, borrow::ToOwned};
use serde::{Serialize, de::DeserializeOwned, Deserialize};

use crate::{tree::node::traits::Node as TNode, arena::ArenaId};

use super::{nodes::traits::Nodes as TNodes, result::TreeResult};

#[derive(Clone)]
pub enum CoreWeakNode<Node>
where Node: TNode
{
    ArenaId(ArenaId),
    Id(Node::Hash)
}

impl<Node> std::cmp::PartialEq for CoreWeakNode<Node>
where Node: TNode
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ArenaId(l0), Self::ArenaId(r0)) => l0 == r0,
            (Self::Id(l0), Self::Id(r0)) => l0 == r0,
            _ => false
        }
    }
}

impl<Node> From<ArenaId> for CoreWeakNode< Node>
where Node: TNode
{
    fn from(arena_id: ArenaId) -> Self {
        Self::ArenaId(arena_id)
    }
}

impl< Node> Default for CoreWeakNode< Node>
where Node: TNode
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

impl<'a, Node> std::ops::Deref for RefMutNode<'a, Node>
{
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, Node> std::ops::DerefMut for RefMutNode<'a, Node>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
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

impl<'a, Node> std::ops::Deref for RefNode<'a, Node>
{
    type Target = Node;

    fn deref(&self) -> &Self::Target {
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
pub struct WeakNode<Node>(Rc<RefCell<CoreWeakNode<Node>>>) where Node: TNode;

impl<Node> Serialize for WeakNode<Node>
where Node: TNode
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let node_id = self.as_node_id().expect("cannot serialize weak node while referring to arena id");
        serializer.serialize_bytes(&node_id.into())
    }
}

impl<'de, Node> Deserialize<'de> for WeakNode<Node>
where Node: TNode
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        Self(Rc::new(RefCell::new(CoreWeakNode::from(arena_id))))
    }
}

impl< Node> std::fmt::Display for WeakNode< Node>
where Node: TNode
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "node::{}", "node")
    }
}

impl< Node> From<ArenaId> for WeakNode< Node>
where Node: TNode
{
    fn from(arena_id: ArenaId) -> Self {
        Self(Rc::new(RefCell::new(CoreWeakNode::from(arena_id))))
    }
}

impl<Node> std::cmp::PartialEq for WeakNode<Node>
where Node: TNode
{
    fn eq(&self, other: &Self) -> bool {
        *self.0.borrow().deref() == *other.0.borrow().deref()
    }
}

impl<Node> ToOwned for WeakNode<Node> 
where Node: TNode
{
    type Owned = Self;

    fn to_owned(&self) -> Self::Owned {
        Self(self.0.clone())
    }

    fn clone_into(&self, target: &mut Self::Owned) {
        *target = self.to_owned();
    }
}


impl< Node> WeakNode< Node>
where Node: TNode
{
    pub fn get_hash<Nodes: TNodes<Node=Node>>(&self, nodes: &Nodes) -> TreeResult<Option<Node::Hash>, Node>
    {
        match self.0.borrow().deref() {
            CoreWeakNode::ArenaId(_) => Ok(self.upgrade(nodes)?.take().get_hash()),
            CoreWeakNode::Id(id) => Ok(Some(id.clone()))
        }
    }

    pub fn upgrade_mut<'a, Nodes: TNodes< Node=Node>>(&self, nodes: &'a mut Nodes) -> TreeResult<RefMutNode<'a, Node>, Node>
    {
        nodes.upgrade_mut(self)
    }

    pub fn upgrade<'a, Nodes: TNodes< Node=Node>>(&self, nodes: &'a Nodes) -> TreeResult<RefNode<'a, Node>, Node>
    {
        nodes.upgrade(self)
    }

    pub fn is_loaded(&self) -> bool {
        if let CoreWeakNode::ArenaId(_) = *self.0.borrow() {
            true
        } else {
            false
        }
    }

    /// Notify the weak node, that the underlying node content has been loaded onto an arena.
    pub fn load(&self, arena_id: ArenaId) 
    {
        *self.0.borrow_mut() = CoreWeakNode::ArenaId(arena_id);
    }

    pub fn unload<Nodes: TNodes<Node=Node>>(&self, nodes: &Nodes)
    {
        let hash = self.upgrade(nodes).get_hash().unwrap();
        *self.0.borrow_mut() = CoreWeakNode::Id(hash)
    }

    pub fn as_arena_id(&self) -> Option<ArenaId>
    {
        match self.0.borrow().deref() {
            CoreWeakNode::ArenaId(id) => Some(*id),
            _ => None
        }
    }

    pub fn as_node_id(&self) -> Option<Node::Hash>
    {
        match self.0.borrow().deref() {
            CoreWeakNode::Id(id) => Some(id.clone()),
            _ => None
        }
    }

}