use super::{TreeRef, Path, NodeType, NodeRef};
use super::node::traits::Node;

/// Search a node in the tree, based on the key
fn search_node<
    'a,
    const SIZE: usize,
    Hash: Clone + PartialEq + Default,
    Key,
    Branch,
    Node,
    Nodes
>(tree: &TreeRef<Hash>, nodes: &'a Nodes, key: &Key) -> Option<&'a Node> 
    where Branch: crate::tree::branch::traits::Branch<SIZE, Hash=Hash, Key=Key>,
          Node: crate::tree::node::traits::Node<SIZE, Hash=Hash, Key=Key, Branch=Branch>, 
          Nodes: crate::tree::node::traits::Nodes<SIZE, Hash=Hash, Key=Key, Node=Node>
{
    let node = search_path(tree, nodes, key)
    .last()
    .and_then(|node_ref| nodes.borrow_node(node_ref));

    node
}

fn search_leaf<
    'a,
    const SIZE: usize,
    Hash: Clone + PartialEq + Default,
    Key,
    Branch: crate::tree::branch::traits::Branch<SIZE, Hash=Hash, Key=Key>,
    Leaf:   crate::tree::leaf::traits::Leaf<SIZE, Key=Key>,
    Node:   crate::tree::node::traits::Node<SIZE, Hash=Hash, Key=Key, Branch=Branch, Leaf=Leaf>,
    Nodes:  crate::tree::node::traits::Nodes<SIZE, Hash=Hash, Key=Key, Node=Node>
>(tree: &TreeRef<Hash>, nodes: &'a Nodes, key: &Key) -> Option<&'a Leaf>
{
    search_node(tree, nodes, key)
    .and_then(|node| node.r#as().as_leaf())
}

/// Search an element in the tree
pub fn search<
    'a,
    const SIZE: usize,
    Hash: Clone + PartialEq + Default,
    Key, Element,
    Leaf:       crate::tree::leaf::traits::Leaf<SIZE, Key=Key, Element=Element>,
    Branch:     crate::tree::branch::traits::Branch<SIZE, Hash=Hash, Key=Key>,
    Node:       crate::tree::node::traits::Node<SIZE, Hash=Hash, Key=Key, Leaf=Leaf, Branch=Branch>,
    Nodes:      crate::tree::node::traits::Nodes<SIZE, Hash=Hash, Key=Key, Node=Node>
>(tree: &TreeRef<Hash>, nodes: &'a Nodes, key: &Key) -> Option<&'a Element>
{
    search_leaf(tree, nodes, key)
    .and_then(|leaf| leaf.search(key))
}

/// Search the element behind the key, if any
fn search_path<
    const SIZE: usize,
    Key,
    Hash: Clone + PartialEq + Default,
    Branch: crate::tree::branch::traits::Branch<SIZE, Hash=Hash, Key=Key>,
    Node: crate::tree::node::traits::Node<SIZE, Key=Key, Branch=Branch>,
    Nodes: crate::tree::node::traits::Nodes<SIZE, Key=Key, Hash=Hash, Node=Node>
>(tree: &TreeRef<Hash>, nodes: &Nodes, key: &Key) -> Path<Hash>
{
    let mut path = Path::<Hash>::default();
    let mut opt_node_ref = tree.get_root();

    while let Some(node_ref) = opt_node_ref 
    {  
        if let Some(node) = nodes.borrow_node(&node_ref) 
        {
            path.push(node_ref.clone());

            match node.r#as() {
                // It's a branch, look for the right child node, if any
                NodeType::Branch(branch) => {
                    let child_node_ref: NodeRef<Hash> = Branch::search_node(branch, key).clone();
                    opt_node_ref = Some(&child_node_ref);
                },
                // Reach a leaf, we cannot go further
                _ => {break;}
            }

        } else {
            break;
        }
    }

    path
}

pub fn insert<
    const SIZE: usize,
    Hash: Clone + PartialEq + Default,
    Key, Element,
    Leaf: crate::tree::leaf::traits::Leaf<SIZE, Key=Key, Element=Element>,
    Branch: crate::tree::branch::traits::Branch<SIZE, Hash=Hash, Key=Key>,
    Node: crate::tree::node::traits::Node<SIZE, Key=Key, Hash=Hash, Leaf=Leaf, Branch=Branch>,
    Nodes: crate::tree::node::traits::Nodes<SIZE, Key=Key, Hash=Hash, Node=Node>
>(tree: &mut TreeRef<Hash>, nodes: &mut Nodes, key: Key, element: Element)
{
    let path = search_path(tree, nodes, &key);

    if let Some(leaf) = path.last()
    .and_then(|node_ref| nodes.borrow_mut_node(node_ref))
    .and_then(|node| node.as_mut().as_mut_leaf()) {
        leaf.insert(key, element);
    } else if path.last().is_none() {
        let node: Node = Leaf::new(key, element).into();
        let node_ref = nodes.allocate(node);
        tree.set_root(Some(node_ref));
    } else {
        panic!("Tree integrity error: expecting to find a leaf to insert element");
    }

    split_if_required(tree, nodes, path);
}

fn split_if_required<
    const SIZE: usize,
    Key,
    Hash: Clone + PartialEq, 
    Branch: crate::tree::branch::traits::Branch<SIZE, Hash=Hash, Key=Key>,
    Node:   crate::tree::node::traits::Node<SIZE, Key=Key, Hash=Hash, Branch=Branch>,
    Nodes:  crate::tree::node::traits::Nodes<SIZE, Node=Node, Hash=Hash>
>(tree: &mut TreeRef<Hash>, nodes: &mut Nodes, mut path: Path<Hash>)
{
while let Some(node_ref) = path.pop()
{
    if let Some(node) = nodes.borrow_mut_node(&node_ref)
    {
        // The node is full, we split it
        if node.is_full() {
            let (key, right_node) = node.split();
            let right_node_ref = nodes.allocate(right_node);
            match path.last()
            .and_then(|node_ref| nodes.borrow_mut_node(node_ref))
            .and_then(|node| node.as_mut().as_mut_branch())
            {
                Some(parent_branch) => {
                    Branch::insert(parent_branch, node_ref.clone(), key, right_node_ref);
                }
                None => {
                    let root_ref = nodes.allocate(
                        Node::from(
                            Branch::new(node_ref.clone(), key, right_node_ref)
                        )
                    );
                    
                    tree.set_root(Some(root_ref));
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
/// Commit the updated tree
/// 1°) Recompute the hashes from loaded nodes from bottom to top
/// 2°) Store the node based on the new hash
/// 3°) If it's a branch, unload all the children ref
/// 4°) Returns the list of updated nodes refs
pub fn commit<
    const SIZE: usize,
    Hash: Clone + PartialEq, 
    Nodes: crate::tree::node::traits::Nodes<SIZE, Hash=Hash>

>(tree: &mut TreeRef<Hash>, nodes: &mut Nodes)  -> Vec<NodeRef<Hash>>
{
    let mut updated_nodes: Vec<NodeRef<Hash>> = vec![];

    if let Some(root_ref) = tree.get_root() {

        while let Some(node_ref) = nodes.get_loaded_nodes(root_ref.clone()).pop_front()
        {
            let mut compute_node_hash: Option<Hash> = None;
            {
                if let Some(node) = nodes.borrow_node(&node_ref)
                {
                    let hash = node.compute_hash(nodes);
                    
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
            
            if let Some(root_ref) = tree.get_root()
            {
                if node_ref == *root_ref {
                    let hash = nodes.borrow_node(&node_ref)
                    .unwrap()
                    .get_hash()
                    .unwrap();
                    
                    tree.set_root(Some(NodeRef::from_key(hash)));
                }
            }
        }
    }

    updated_nodes
}