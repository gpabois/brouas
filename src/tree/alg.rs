use super::error::TreeError;
use super::node_ref::{RefMutNode, RefNode, WeakNode};
use super::nodes::traits::Nodes as TNodes;
use super::result::TreeResult;
use crate::tree::node::traits::Node as TNode;
use super::{Tree, Path, NodeType};
use crate::tree::branch::traits::Branch;
use crate::tree::leaf::traits::Leaf;

/// Search a node in the tree, based on the key
fn search_node<'a, Nodes>(
    tree: &Tree< Nodes::Node>, 
    nodes: &'a Nodes, 
    key: &<Nodes::Node as TNode>::Key) -> TreeResult< Option<RefNode<'a, <Nodes as TNodes>::Node>>, <Nodes as TNodes>::Node>
    where Nodes: TNodes
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
    tree: &mut Tree< Nodes::Node>, 
    nodes: &'a mut Nodes, 
    key: &<Nodes::Node as TNode>::Key
) -> TreeResult< Option<RefMutNode<'a, Nodes::Node>>, Nodes::Node>
    where Nodes: TNodes
{
    if let Some(weak_node) = search_path(tree, nodes, key)?.last() {
        Ok(Some(weak_node.upgrade_mut(nodes)?))
    } else {
        Ok(None)
    }
}

fn search_mut_leaf<'a, Nodes>(
    tree: &mut Tree< Nodes::Node>, 
    nodes: &'a mut Nodes, 
    key: &<Nodes::Node as TNode>::Key) -> TreeResult< &'a mut <Nodes::Node as TNode>::Leaf, Nodes::Node>
    where Nodes: TNodes
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
    tree: &Tree< Nodes::Node>, 
    nodes: &'a Nodes, 
    key: &<Nodes::Node as TNode>::Key
) -> TreeResult< &'a <Nodes::Node as TNode>::Leaf, Nodes::Node>
where Nodes: TNodes
{
    if let Some(leaf )= search_node(tree, nodes, key)?.and_then(|node| node.take().r#as().as_leaf()) {
        Ok(leaf)
    } else {
        Err(TreeError::ExpectingLeaf)
    }
}

pub fn search_mut<'a, Nodes>(
    tree: &mut Tree< Nodes::Node>, 
    nodes: &'a mut Nodes, 
    key: &<Nodes::Node as TNode>::Key
) -> TreeResult< Option<&'a mut <Nodes::Node as TNode>::Element>, Nodes::Node>
where Nodes: TNodes
{
    Ok(search_mut_leaf(tree, nodes, key)?.search_mut(key))
}

/// Search an element in the tree
pub fn search<'a, Nodes>(
    tree: &Tree< Nodes::Node>, 
    nodes: &'a Nodes, 
    key: &<Nodes::Node as TNode>::Key) -> TreeResult< Option<&'a <Nodes::Node as TNode>::Element>, Nodes::Node>
    where Nodes: TNodes
{
    Ok(search_leaf(tree, nodes, key)?.search(key))
}

/// Search the element behind the key, if any
fn search_path<'a, Nodes>(
    tree: &Tree< Nodes::Node>, 
    nodes: &'a Nodes, 
    key: &<Nodes::Node as TNode>::Key) -> TreeResult<Path<Nodes::Node>, Nodes::Node>
where Nodes: TNodes
{
    let mut path = Path::< Nodes::Node>::new();
    let mut opt_node_ref = tree.get_root().to_owned();

    while let Some(node_ref) = opt_node_ref
    {  
        path.push(node_ref.to_owned());

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
    tree: &mut Tree< Nodes::Node>, 
    nodes: &'a mut Nodes, 
    key: <Nodes::Node as TNode>::Key, 
    element: <Nodes::Node as TNode>::Element) -> TreeResult< (), Nodes::Node>
where Nodes: TNodes
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

fn insert_to_parent_or_update_root< Nodes>(
    tree: &mut Tree< Nodes::Node>,
    nodes: &mut Nodes,
    parent: Option<&WeakNode<Nodes::Node>>, 
    place: &WeakNode<Nodes::Node>,
    left: WeakNode<Nodes::Node>,
    key: <Nodes::Node as TNode>::Key,
    right: WeakNode< Nodes::Node>
) -> TreeResult< (), Nodes::Node>
where Nodes: TNodes {
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
            .take()
            .as_mut()
            .as_mut_branch()
            .ok_or(TreeError::<Nodes::Node>::ExpectingBranch)?;
            branch.insert(place, left, key, right);
            Ok(())
        }
    }
}

fn bottom_up_uncommitted_nodes<Nodes>(tree: &Tree<Nodes::Node>, nodes: &Nodes) -> TreeResult<Vec<(usize, WeakNode<Nodes::Node>)>, Nodes::Node>
where Nodes: TNodes
{
    let mut updated_nodes: Vec<(usize, WeakNode<Nodes::Node>)> = vec![];
    let mut stack: Vec<_> = tree.get_root().iter().cloned().map(|w| (0, w)).collect();

    while let Some((depth, weak_node)) = stack.pop()
    {
        if weak_node.is_loaded() {
            updated_nodes.push((depth, weak_node.to_owned()));
            let node_ref = nodes.upgrade(weak_node)?.take();
            
            node_ref
            .children()
            .iter()
            .for_each(|ref_weak| {
                stack.push((depth + 1, ref_weak.clone()))
            });
        }

    }

    updated_nodes.sort_unstable_by(|a, b| {
        a.0.cmp(&b.0)
    });
    updated_nodes.reverse();

    Ok(updated_nodes)
}

pub fn calculate_hashes<Nodes>(tree: &Tree<Nodes::Node>, nodes: &mut Nodes) 
-> TreeResult<Option<<Nodes::Node as TNode>::Hash>, Nodes::Node>
where Nodes: TNodes
{
    let mut bu_weak_nodes = bottom_up_uncommitted_nodes(tree, nodes)?;

    while let Some((_, weak_node)) = bu_weak_nodes.pop() 
    {
        let node = weak_node.upgrade(nodes)?.take();
        
        if node.get_hash().is_none() {
            let hash = node.compute_hash(nodes)?;
            weak_node.upgrade_mut(nodes)?.take().set_hash(hash);
        }

    }

    if let Some(weak) = tree.get_root() {
        weak.get_hash(nodes)
    } else {
        Ok(None)
    }
}

fn split_if_required< Nodes>(
    tree: &mut Tree< Nodes::Node>, 
    nodes: &mut Nodes, 
    mut path: Path< Nodes::Node>) -> TreeResult< (), Nodes::Node>
    where Nodes: TNodes
{
    while let Some(weak_node) = path.pop()
    {        
        let node = weak_node.upgrade_mut(nodes)?.take();

        // The node is full, we split it
        if node.is_full() {
            let (
                left_node, 
                key, 
                right_node
            ) = node.split();
            
            let right = nodes.alloc(right_node);
            let left = nodes.alloc(left_node);
            
            insert_to_parent_or_update_root(tree, nodes, path.last(), &weak_node, left, key, right)?;
        } else {
            return Ok(())
        } 
    }

    Ok(())
}