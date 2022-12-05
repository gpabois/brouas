use std::ops::DerefMut;

use super::error::TreeError;
use super::node_ref::{RefMutNode, RefNode, WeakNode};
use super::nodes::traits::Nodes as TNodes;
use super::result::TreeResult;
use crate::tree::node::traits::Node as TNode;
use super::{Tree, Path, NodeType, MutPath};
use crate::tree::branch::traits::Branch;
use crate::tree::leaf::traits::Leaf;

/// Search a node in the tree, based on the key
fn search_node<'a, Nodes>(
    tree: &'a Tree<'a, Nodes::Node>, 
    nodes: &'a Nodes, 
    key: &<Nodes::Node as TNode<'a>>::Key) -> TreeResult<'a, Option<RefNode<'a, <Nodes as TNodes<'a>>::Node>>, <Nodes as TNodes<'a>>::Node>
    where Nodes: TNodes<'a>
{
    let mut path =  search_path(tree, nodes, key)?;
    if let Some(weak_node) = path.pop() {
        let node = weak_node.upgrade(nodes)?;
        Ok(Some(node))
    } else {
        Ok(None)
    }
}

fn search_mut_node<'a, Nodes>(
    tree: &'a mut Tree<'a, Nodes::Node>, 
    nodes: &'a Nodes, 
    key: &<Nodes::Node as TNode<'a>>::Key
) -> TreeResult<'a, Option<RefMutNode<'a, Nodes::Node>>, Nodes::Node>
    where Nodes: TNodes<'a>
{
    if let Some(weak_node) = search_mut_path(tree, nodes, key)?.last() {
        Ok(Some(weak_node.upgrade_mut(nodes)?))
    } else {
        Ok(None)
    }
}

fn search_mut_leaf<'a, Nodes>(
    tree: &'a mut Tree<'a, Nodes::Node>, 
    nodes: &'a Nodes, 
    key: &<Nodes::Node as TNode<'a>>::Key) -> TreeResult<'a, &'a mut <Nodes::Node as TNode<'a>>::Leaf, Nodes::Node>
    where Nodes: TNodes<'a>
{
    if let Some(leaf )= search_mut_node(tree, nodes, key)?
    .and_then(|node| 
        node
        .take()
        .as_mut()
        .as_mut_leaf()
    ) {
        Ok(leaf)
    } else {
        Err(TreeError::ExpectingLeaf)
    }
}

fn search_leaf<'a, Nodes>(
    tree: &'a Tree<'a, Nodes::Node>, 
    nodes: &'a Nodes, 
    key: &<Nodes::Node as TNode<'a>>::Key
) -> TreeResult<'a, &'a <Nodes::Node as TNode<'a>>::Leaf, Nodes::Node>
where Nodes: TNodes<'a>
{
    if let Some(leaf )= search_node(tree, nodes, key)?.and_then(|node| node.take().r#as().as_leaf()) {
        Ok(leaf)
    } else {
        Err(TreeError::ExpectingLeaf)
    }
}

pub fn search_mut<'a, Nodes>(
    tree: &'a mut Tree<'a, Nodes::Node>, 
    nodes: &'a Nodes, 
    key: &<Nodes::Node as TNode<'a>>::Key
) -> TreeResult<'a, Option<&'a mut <Nodes::Node as TNode<'a>>::Element>, Nodes::Node>
where Nodes: TNodes<'a>
{
    Ok(search_mut_leaf(tree, nodes, key)?.search_mut(key))
}

/// Search an element in the tree
pub fn search<'a, Nodes>(
    tree: &'a Tree<'a, Nodes::Node>, 
    nodes: &'a Nodes, 
    key: &<Nodes::Node as TNode<'a>>::Key) -> TreeResult<'a, Option<&'a <Nodes::Node as TNode<'a>>::Element>, Nodes::Node>
    where Nodes: TNodes<'a>
{
    Ok(search_leaf(tree, nodes, key)?.search(key))
}

/// Search the element behind the key, if any
fn search_mut_path<'a, Nodes>(
    tree: &'a mut Tree<'a, Nodes::Node>, 
    nodes: &'a Nodes, 
    key: &<Nodes::Node as TNode<'a>>::Key) -> TreeResult<'a, MutPath<'a, Nodes::Node>, Nodes::Node>
where Nodes: TNodes<'a>
{
    let mut path = MutPath::<'a, Nodes::Node>::new();
    let mut opt_node_ref = tree.get_mut_root();

    while let Some(node_ref) = opt_node_ref
    {  
        path.push(node_ref);

        let node = node_ref.upgrade_mut(nodes)?;
        
        match node.take().as_mut() {
            // It's a branch, look for the right child node, if any
            NodeType::Branch(branch) => {
                let child_node_ref = Branch::search_mut_node(branch, key);
                opt_node_ref = Some(child_node_ref);
            },
            // Reach a leaf, we cannot go further
            _ => {break;}
        }
    }

    Ok(path)
}

/// Search the element behind the key, if any
fn search_path<'a, Nodes>(
    tree: &'a Tree<'a, Nodes::Node>, 
    nodes: &'a Nodes, 
    key: &<Nodes::Node as TNode<'a>>::Key) -> TreeResult<'a, Path<'a, Nodes::Node>, Nodes::Node>
where Nodes: TNodes<'a>
{
    let mut path = Path::<'a, Nodes::Node>::new();
    let mut opt_node_ref = tree.get_root();

    while let Some(node_ref) = opt_node_ref
    {  
        path.push(node_ref);

        let node = node_ref.upgrade(nodes)?;
        
        match node.take().r#as() {
            // It's a branch, look for the right child node, if any
            NodeType::Branch(branch) => {
                let child_node_ref = Branch::search_node(branch, key).clone();
                opt_node_ref = Some(child_node_ref);
            },
            // Reach a leaf, we cannot go further
            _ => {break;}
        }
    }

    Ok(path)
}

pub fn insert<'a, Nodes>(
    tree: &mut Tree<'a, Nodes::Node>, 
    nodes: &Nodes, 
    key: <Nodes::Node as TNode<'a>>::Key, 
    element: <Nodes::Node as TNode<'a>>::Element) -> TreeResult<'a, (), Nodes::Node>
where Nodes: TNodes<'a>
{
    let path = search_path(tree, nodes, &key)?;

    if let Some(weak_node) = path.last() {
        let leaf = weak_node.upgrade_mut(nodes)?
            .take()
            .as_mut()
            .as_mut_leaf()
            .ok_or(TreeError::MissingLeaf)?;
        
        
        leaf.insert(key, element);
    } else if path.last().is_none() {
        let node: Nodes::Node = <Nodes::Node as TNode>::Leaf::new(key, element).into();
        let node_ref = nodes.alloc(node);
        tree.set_root(Some(node_ref));
    } else {
        return Err(TreeError::MissingLeaf);
    }

    split_if_required(tree, nodes, path)?;

    Ok(())
}

fn insert_to_parent_or_update_root<'a, Nodes>(
    tree: &mut Tree<'a, Nodes::Node>,
    nodes: &Nodes,
    parent: Option<&&WeakNode<'a, Nodes::Node>>, 
    place: &WeakNode<'a, Nodes::Node>,
    left: WeakNode<'a, Nodes::Node>,
    key: <Nodes::Node as TNode<'a>>::Key,
    right: WeakNode<'a, Nodes::Node>
) -> TreeResult<'a, (), Nodes::Node>
where Nodes: TNodes<'a> {
    match parent {
        None => {
            let root_ref = nodes.alloc(
                <Nodes::Node as TNode>::Branch::new(left, key, right).into()
            );
            
            tree.set_root(Some(root_ref));    
            Ok(())   
        },
        Some(weak_node) => {
            let branch = weak_node.upgrade_mut(nodes)?
            .take().as_mut().as_mut_branch().ok_or(TreeError::<Nodes::Node>::ExpectingBranch)?;
            branch.insert(place, left, key, right);
            Ok(())
        }
    }
}

fn split_if_required<'a, Nodes>(
    tree: &mut Tree<'a, Nodes::Node>, 
    nodes: &'a Nodes, 
    mut path: Path<'a, Nodes::Node>) -> TreeResult<'a, (), Nodes::Node>
    where Nodes: TNodes<'a>
{
    while let Some(weak_node) = path.pop()
    {        
        let mut node = weak_node.upgrade_mut(nodes)?.take();

        // The node is full, we split it
        if node.is_full() {
            let (
                left_node, 
                key, 
                right_node
            ) = node.split();
            
            let right = nodes.alloc(right_node);
            let left = nodes.alloc(left_node);
            
            insert_to_parent_or_update_root(tree, nodes, path.last(), weak_node, left, key, right)?;
        } else {
            return Ok(())
        } 
    }

    Ok(())
}