use std::fmt::Display;

use super::Node;
use super::node::traits::Node as TNode;
use super::node_ref::WeakNode;

#[derive(Default)]
pub struct Path<Node>(Vec<WeakNode<Node>>)
    where Node: TNode;

impl<Node> Path<Node>
    where Node: TNode
{
    pub fn new() -> Self {
        Self(vec![])
    }
    
    pub fn last(&self) -> Option<&WeakNode< Node>>
    {
        self.0.last()
    }

    pub fn push(&mut self, node_ref: WeakNode<Node>)
    {
        self.0.push(node_ref);
    }

    pub fn pop(&mut self) -> Option<WeakNode<Node>>
    {
        self.0.pop()
    }
}

#[derive(Default)]
pub struct Tree< Node> where Node: TNode {
    root: Option<WeakNode< Node>>
}

impl< Node> Display for Tree< Node> 
where Node: TNode
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(root) = &self.root {
            write!(f, "tree::{}", root)
        } else {
            write!(f, "tree::empty")
        }
        
    }
}

impl< Node> std::fmt::Debug for Tree< Node> 
where Node: TNode
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

/// Create a new Merkle B+ Tree
pub fn new_merkle_bp_tree<const SIZE: usize, Hash, Key, Element> () -> Tree<Node<SIZE, Hash, Key, Element>>
{
    Tree::empty()
} 

impl<Node> Tree<Node>
where Node: TNode
{
    pub fn empty() -> Self {
        Self {root: None}
    }

    pub fn existing(root: WeakNode<Node>) -> Self {
        Self {root: Some(root)}
    }

    pub fn set_root(&mut self, root: Option<WeakNode< Node>>)
    {
        self.root = root;
    }

    pub fn get_root(&self) -> Option<&WeakNode< Node>>
    {
        return self.root.as_ref();
    }
}
