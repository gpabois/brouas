
pub enum NodeType<const SIZE: usize, Branch, Leaf>
where Branch: crate::tree::branch::traits::Branch<SIZE>,
      Leaf: crate::tree::leaf::traits::Leaf<SIZE>
{
    Leaf(Leaf),
    Branch(Branch)
}

impl<const SIZE: usize, Branch, Leaf> NodeType<SIZE, Branch, Leaf>
where Branch: crate::tree::branch::traits::Branch<SIZE>,
      Leaf: crate::tree::leaf::traits::Leaf<SIZE>
{
    pub fn as_branch(&self) -> Option<&Branch>
    {
        match self {
            NodeType::Branch(branch) => Some(branch),
            _ => None
        }
    }
    pub fn as_leaf(&self) -> Option<&Leaf>
    {
        match self {
            NodeType::Leaf(leaf) => Some(leaf),
            _ => None
        }
    }

    pub fn as_mut_branch(&mut self) -> Option<&mut Branch>
    {
        match self {
            NodeType::Branch(branch) => Some(branch),
            _ => None
        }
    }
    pub fn as_mut_leaf(&mut self) -> Option<&mut Leaf>
    {
        match self {
            NodeType::Leaf(leaf) => Some(leaf),
            _ => None
        }
    }
}

pub mod traits {
    use std::collections::VecDeque;

    use crate::{tree::NodeRef, arena::Allocator};

    use super::NodeType;

    pub trait BorrowNode<Hash: Clone + PartialEq, Node>
    {
        fn borrow_node<'a>(&'a self, node_ref: &NodeRef<Hash>) -> Option<&'a Node>;
    }
    
    pub trait BorrowMutNode<Hash: Clone + PartialEq, Node>
    {
        fn borrow_mut_node<'a>(&'a self, node_ref: &NodeRef<Hash>) -> Option<&'a mut Node>;
    }
    
    /// Nodes collection trait
    pub trait Nodes<const SIZE: usize>: BorrowMutNode<Self::Hash, Self::Node> + BorrowNode<Self::Hash, Self::Node> + Allocator<Self::Hash, Self::Node>
    {
        type Hash: Clone + PartialEq;

        type Leaf: crate::tree::leaf::traits::Leaf<SIZE, Hash=Self::Hash, Key=Self::Key>;
        type Branch: crate::tree::branch::traits::Branch<SIZE, Key=Self::Key, Node=Self>;
        type Key;
        type Node: Node<SIZE, Hash=Self::Hash, Key=Self::Key, Leaf=Self::Leaf, Branch=Self::Branch>;

        /// Returns the loaded nodes, from bottom to top, starting at the top
        fn get_loaded_nodes(&self, root_ref: NodeRef<Self::Hash>) -> VecDeque<NodeRef<Self::Hash>>
        {
            let mut loaded_nodes: Vec<(usize, NodeRef<Self::Hash>)> = Default::default();
            let mut queue: VecDeque<(usize, NodeRef<Self::Hash>)> = Default::default();
            
            queue.push_back((0, root_ref));
            
            while let Some((depth, node_ref)) = queue.pop_front()
            {
                if node_ref.is_loaded()
                {
                    let node = self.borrow_node(&node_ref).unwrap();
                    
                    let mut children_ref: VecDeque<(usize, NodeRef<Self::Hash>)> = node
                    .children_ref()
                    .into_iter()
                    .cloned()
                    .map(|c| (depth + 1, c))
                    .collect();

                    queue.append(&mut children_ref);

                } else {
                    loaded_nodes.push((depth, node_ref))
                }
            }

            loaded_nodes.sort_unstable_by_key(|(d, _)| *d);
            loaded_nodes.reverse();

            loaded_nodes.into_iter().map(|(_, n)| n).collect()
        }

    }

    /// The MBT Node Trait
    pub trait Node<const SIZE: usize>: From<Self::Leaf> + From<Self::Branch> + PartialEq<Self::Hash>
    {
        type Key;

        type Hash: Clone + PartialEq;

        type Leaf: crate::tree::leaf::traits::Leaf<SIZE, Hash=Self::Hash, Key=Self::Key>;
        type Branch: crate::tree::branch::traits::Branch<SIZE, Key=Self::Key, Node=Self>;

        fn r#as(&self) -> &NodeType<SIZE, Self::Branch, Self::Leaf>;
        fn as_mut(&self) -> &mut NodeType<SIZE, Self::Branch, Self::Leaf>;

        fn children_ref<'a>(&'a self) -> Vec<&'a NodeRef<Self::Hash>>;

        fn compute_hash<Nodes: BorrowNode<Self::Hash, Self>>(&self, nodes: &Nodes) -> Self::Hash;
        
        fn set_hash(&mut self, hash: Self::Hash);
        fn get_hash(&self) -> Option<Self::Hash>;

        fn split(&self) -> (Self::Key, Self);
        fn is_full(&self) -> bool;
    }

}


