use super::NodeRef;

#[derive(Default)]
pub struct Path<Hash: Clone + PartialEq>(Vec<NodeRef<Hash>>);

impl<Hash: Clone+PartialEq> Path<Hash>
{
    pub fn last<'a>(&'a self) -> Option<&'a NodeRef<Hash>>
    {
        self.0.last()
    }

    pub fn push(&mut self, node_ref: NodeRef<Hash>)
    {
        self.0.push(node_ref);
    }

    pub fn pop(&mut self) -> Option<NodeRef<Hash>>
    {
        self.0.pop()
    }
}

#[derive(Default)]
pub struct TreeRef<Hash: Clone + PartialEq>{
    root: Option<NodeRef<Hash>>
}

impl<Hash: Clone + PartialEq> TreeRef<Hash>
{
    pub fn set_root(&mut self, root: Option<NodeRef<Hash>>)
    {
        self.root = root;
    }

    pub fn get_root<'a>(&'a self) -> Option<&'a NodeRef<Hash>>
    {
        return self.root.as_ref();
    }
}

/// Merkle B+ Tree
pub struct MPBTree<const SIZE: usize, Hash, Nodes> 
where Hash: Clone + PartialEq, Nodes: crate::tree::node::traits::Nodes<SIZE>
{
    tree_ref: TreeRef<Hash>,
    node: Nodes
}


