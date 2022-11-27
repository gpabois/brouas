use std::collections::VecDeque;

use crate::arena::{Allocator};
use crate::arena::Arena;

use super::{NodeRef, Leaf, Branch, Node, Nodes, NodeType, BorrowNode, BorrowMutNode};

#[derive(Default)]
pub struct Path<Hash: Clone + PartialEq>(Vec<NodeRef<Hash>>);

impl<Hash: Clone+PartialEq> Path<Hash>
{
    pub fn last<'a>(&'a self) -> Option<&'a NodeRef<Hash>>
    {
        self.0.last()
    }

    pub fn pop(&mut self) -> Option<NodeRef<Hash>>
    {
        self.0.pop()
    }
}

pub trait Tree<const SIZE: usize>: BorrowNode<Self::Hash, Self::Node> + BorrowMutNode<Self::Hash, Self::Node> + Allocator<Self::Hash, Self::Node>
{
    type Hash: Clone + PartialEq;
    type Key;
    type Element;

    type Arena: crate::arena::Arena<Key=Self::Hash, Value=Self::Node>;
    type Storage: crate::storage::Storage<Key=Self::Hash, Value=Self::Node>;

    type Leaf: Leaf<SIZE, Hash=Self::Hash, Key=Self::Key, Element=Self::Element>;
    type Branch: Branch<SIZE, Hash=Self::Hash, Key=Self::Key, Node=Self::Node>;
    type Node: Node<SIZE, Hash=Self::Hash, Key=Self::Key, Leaf=Self::Leaf, Branch=Self::Branch>;

    fn get_root(&self) -> Option<NodeRef<Self::Hash>>;
    fn set_root<'a>(&'a mut self, root: Option<NodeRef<Self::Hash>>);

    /// Commit the updated tree
    /// 1°) Recompute the hashes from loaded nodes from bottom to top
    /// 2°) Store the node based on the new hash
    /// 3°) If it's a branch, unload all the children ref
    /// 4°) Returns the list of updated nodes refs
    fn commit_from_nodes<'b, N: Nodes<Self::Hash, Self::Node>>(&'b mut self, nodes: &'b mut N) -> Vec<NodeRef<Self::Hash>> where Self: Sized
    {
        let mut updated_nodes: Vec<NodeRef<Self::Hash>> = vec![];

        if let Some(root_ref) = self.get_root() {

            while let Some(node_ref) = nodes.get_loaded_nodes(root_ref).pop_front()
            {
                let mut compute_node_hash: Option<Self::Hash> = None;
                {
                    if let Some(node) = nodes.borrow_node(&node_ref)
                    {
                        let hash = node.compute_hash(self);
                        
                        if *node != hash {
                            compute_node_hash = Some(hash);
                            updated_nodes.push(node_ref.clone());
                        }
                        
                    }

                    // If we need to set the new branch hash
                    if let Some(node_hash) = compute_node_hash 
                    {
                        if let Some(node) = nodes.borrow_mut_node(&node_ref)
                        {
                            node.set_hash(node_hash)
                        }
                    }
                }

                // Now we unload the children ref
                if let Some(node) = nodes.borrow_mut_node(&node_ref) 
                {
                    node
                    .children_ref()
                    .iter()
                    .map(|cr|(cr, nodes.borrow_node(*cr)))
                    .filter(|(_cr, c)| c.is_some())
                    .map(|(cr, c)| (cr, c.unwrap()))
                    .for_each(|(cr, c)| cr.unload(c.get_hash().unwrap()))
                }
                
                if let Some(root_ref) = self.get_root()
                {
                    if node_ref == root_ref {
                        let hash = nodes.borrow_node(&node_ref)
                        .unwrap()
                        .get_hash()
                        .unwrap();
                        
                        self.set_root(Some(NodeRef::from_key(hash)));
                    }
                }
            }
        }
        
        updated_nodes
    }

    // Insert an element in the tree
    fn insert_from_nodes<N: Nodes<Self::Hash, Self::Node>>(&mut self, key: Self::Key, element: Self::Element, nodes: &mut N)
    {
        let path = self.search_path(&key);
        
        if let Some(leaf) = path.last()
        .and_then(|node_ref| self.borrow_mut_node(node_ref))
        .and_then(|node| node.as_mut().as_mut_leaf()) {
            leaf.insert(key, element);
        } else if path.last().is_none() {
            let node: Self::Node = Self::Leaf::new(key, element).into();
            let node_ref = self.allocate(node);
            self.set_root(Some(node_ref));
        } else {
            panic!("Tree integrity error: expecting to find a leaf to insert element");
        }

        self.split_if_required(path);
    }

    fn split_if_required(&mut self, mut path: Path<Self::Hash>)
    {
        while let Some(node_ref) = path.pop()
        {
            if let Some(node) = self.borrow_mut_node(&node_ref)
            {
                // The node is full, we split it
                if node.is_full() {
                    let (key, right_node) = node.split();
                    let right_node_ref = self.allocate(right_node);
                    match path.last()
                    .and_then(|node_ref| self.borrow_mut_node(node_ref))
                    .and_then(|node| node.as_mut().as_mut_branch())
                    {
                        Some(parent_branch) => {
                            parent_branch.insert(node_ref.clone(), key, right_node_ref);
                        }
                        None => {
                            let root_ref = self.allocate(
                                Self::Node::from(
                                    Self::Branch::new(node_ref.clone(), key, right_node_ref)
                                )
                            );
                            
                            self.set_root(Some(root_ref));
                        }
                    }
                } else {
                    return;
                }
            } else {
                panic!("Tree integrity error: expecting a node behind the given reference.");
            }
        }
    }
    
    // Search the element behind the key, if any
    fn search_path<'a>(&'a self, key: &Self::Key) -> Path<Self::Hash>
    {
        let mut path = Path::<Self::Hash>(vec![]);
        let mut opt_node_ref = self.get_root();

        while let Some(node_ref) = opt_node_ref 
        {  
            if let Some(node) = self.borrow_node(&node_ref) 
            {
                path.0.push(node_ref.clone());

                match node.r#as() {
                    NodeType::Branch(branch) => {
                        opt_node_ref = Some(branch.search_node(key).clone());
                    },
                    _ => {break;}
                }

            } else {
                break;
            }
        }

        path
    }

    fn search_node<'a>(&'a self, key: &Self::Key) -> Option<&'a Self::Node>
    {
        self.search_path(key)
        .last()
        .and_then(|node_ref| self.borrow_arena().borrow_element(node_ref))
    }

    fn search_leaf<'a>(&'a self, key: &Self::Key) -> Option<&'a Self::Leaf>
    {
        self.search_node(key).and_then(|node| node.r#as().as_leaf())
    }
    
    fn search<'a>(&'a self, key: &Self::Key) -> Option<&'a Self::Element>
    {
        self.search_leaf(key).and_then(|leaf| leaf.search(key))
    }
}
