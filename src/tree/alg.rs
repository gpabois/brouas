use std::ops::DerefMut;

use super::error::TreeError;
use super::node_ref::NodeRef;
use super::nodes::traits::Nodes as TNodes;
use super::result::TreeResult;
use crate::tree::node::traits::Node as TNode;
use super::{Tree, Path, NodeType};
use crate::tree::branch::traits::Branch;
use crate::tree::leaf::traits::Leaf;


/// Search a node in the tree, based on the key
fn search_node<'a, Nodes>(
    tree: &Tree<<Nodes as TNodes>::Node>, 
    nodes: &Nodes, 
    key: &<<Nodes as TNodes>::Node as TNode>::Key) -> TreeResult<Option<&'a NodeRef<'a, <Nodes as TNodes>::Node>>, <Nodes as TNodes>::Node>
    where Nodes: TNodes
{
    Ok(search_path(tree, nodes, key)?.pop())
}

fn search_mut_node<'a, Nodes>(
    tree: &Tree<<Nodes as TNodes>::Node>, 
    nodes: &'a Nodes, 
    key: &<<Nodes as TNodes>::Node as TNode>::Key
) -> TreeResult<Option<&'a mut <Nodes as TNodes>::Node>, <Nodes as TNodes>::Node>
    where Nodes: TNodes
{
    Ok(search_path(tree, nodes, key)?.last()
    .map(|node_ref| node_ref.deref_mut()))

}

fn search_mut_leaf<'a, Nodes>(
    tree: &Tree<<Nodes as TNodes>::Node>, 
    nodes: &'a Nodes, 
    key: &<<Nodes as TNodes>::Node as TNode>::Key) -> TreeResult<&'a mut <<Nodes as TNodes>::Node as TNode>::Leaf, <Nodes as TNodes>::Node>
    where Nodes: TNodes
{
    if let Some(leaf )= search_mut_node(tree, nodes, key)?.and_then(|node| node.as_mut().as_mut_leaf()) {
        Ok(leaf)
    } else {
        Err(TreeError::ExpectingLeaf)
    }
}

fn search_leaf<'a, Nodes>(
    tree: &Tree<Nodes::Node>, 
    nodes: &'a Nodes, 
    key: &<<Nodes as TNodes>::Node as TNode>::Key
) -> TreeResult<&'a <Nodes::Node as TNode>::Leaf, <Nodes as TNodes>::Node>
where Nodes: TNodes
{
    if let Some(leaf )= search_node(tree, nodes, key)?.and_then(|node| node.r#as().as_leaf()) {
        Ok(leaf)
    } else {
        Err(TreeError::ExpectingLeaf)
    }
}

pub fn search_mut<'a, Nodes>(
    tree: &Tree<Nodes::Node>, 
    nodes: &'a Nodes, 
    key: &<<Nodes as TNodes>::Node as TNode>::Key
) -> TreeResult<Option<&'a mut <<Nodes as TNodes>::Node as TNode>::Element>, <Nodes as TNodes>::Node>
where Nodes: TNodes
{
    Ok(search_mut_leaf(tree, nodes, key)?.search_mut(key))
}

/// Search an element in the tree
pub fn search<'a, Nodes>(
    tree: &Tree<Nodes::Node>, 
    nodes: &'a Nodes, 
    key: &<Nodes::Node as TNode>::Key) -> TreeResult<Option<&'a <Nodes::Node as TNode>::Element>, <Nodes as TNodes>::Node>
    where Nodes: TNodes
{
    Ok(search_leaf(tree, nodes, key)?.search(key))
}

/// Search the element behind the key, if any
fn search_path<'a, Nodes>(tree: &Tree<Nodes::Node>, nodes: &Nodes, key: &<<Nodes as TNodes>::Node as TNode>::Key) -> TreeResult<Path<'a, Nodes::Node>, <Nodes as TNodes>::Node>
where Nodes: TNodes
{
    let mut path = Path::<'a, <Nodes as TNodes>::Node>::new();
    let stack: Vec<_> = tree.get_root().into_iter().collect();
    nodes.load_nodes_if_not(stack.into_iter());
    
    let mut opt_node_ref = tree.get_root();

    while let Some(node_ref) = opt_node_ref
    {  
        path.push(node_ref);

        match node_ref.r#as() {
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

pub fn insert<'a, Nodes>(tree: &mut Tree<'a, Nodes::Node>, nodes: &mut Nodes, key: <<Nodes as TNodes>::Node as TNode>::Key, element: <<Nodes as TNodes>::Node as TNode>::Element) -> Result<(), TreeError<Nodes::Node>>
where Nodes: TNodes
{
    let path = search_path(tree, nodes, &key)?;

    if let Some(leaf) = path.last()
    .and_then(|node| node.as_mut().as_mut_leaf()) {
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

fn split_if_required<'a, Nodes>(tree: &mut Tree<Nodes::Node>, nodes: &Nodes, mut path: Path<'a, Nodes::Node>) -> Result<(), TreeError<Nodes::Node>>
    where Nodes: TNodes
{
    while let Some(node) = path.pop()
    {        
        // The node is full, we split it
        if node.is_full() {
            let (
                left_node, 
                key, 
                right_node
            ) = (*node).split();
            
            let right_node = nodes.alloc(right_node);
            let left_node = nodes.alloc(left_node);
            
            match path.last()
            .and_then(|node| node.as_mut().as_mut_branch())
            {
                Some(parent_branch) => {
                    <Nodes::Node as TNode>::Branch::insert(parent_branch, node, left_node, key, right_node);
                }
                None => {
                    let root_ref = nodes.alloc(
                        <Nodes::Node as TNode>::Branch::new(left_node, key, right_node).into()
                    );
                    
                    tree.set_root(Some(root_ref));
                }
            }
        } else {
            return Ok(());
        } 
    }

    Ok(())
}