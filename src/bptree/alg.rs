use super::{node::BPTreeNodeId, nodes::traits::{BPTreeNodes, Split}, BPTree};

pub type Path = Vec<BPTreeNodeId>;

pub fn search_path<Nodes: BPTreeNodes>(tree: &BPTree, nodes: &Nodes, key: &Nodes::Key) -> Path {
    let mut path = Path::default();
    let mut cursor = tree.get_root();

    while let Some(node) = cursor {
        path.push(node);

        if nodes.is_leaf(node) {
            break;
        } else {
            cursor = nodes.branch_search(node, key);
        }
    }

    path
}

fn balance_overflow<Nodes: BPTreeNodes>(tree: &mut BPTree, nodes: &mut Nodes, mut path: Path) {
    let mut opt_split: Option<Split<Nodes::Key>> = None;
    
    while let Some(tail) = path.pop() {
        if let Some(split) = opt_split {
            nodes.branch_insert(tail, split);
            opt_split = None;
        }

        if nodes.is_overflowing(tail) {
            opt_split = Some(nodes.split(tail))
        } else {
            break;
        }
    }
    
    if path.is_empty() && opt_split.is_some() {
        tree.set_root(
            Some(
                nodes.new_branch(tree.get_capacity(), opt_split.unwrap())
            )
        )
    }
}

/// Insert a key, value tuple in the tree.
pub fn insert<Nodes: BPTreeNodes>(tree: &mut BPTree, nodes: &mut Nodes, key: Nodes::Key, value: Nodes::Value) {
    let mut path = search_path(tree, nodes, &key);

    // Tree empty
    if path.is_empty() {
        tree.set_root(
            Some(
                nodes.new_leaf(tree.get_capacity(), key, value)
            )
        );
    } else {
        let leaf = path.pop().unwrap();
        nodes.leaf_insert(leaf, key, value);
        
        // Handle the overflow, and balance the tree accordingly
        balance_overflow(tree, nodes, path);
    }
}
