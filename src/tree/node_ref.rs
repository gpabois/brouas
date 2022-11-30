use crate::arena::traits::TLElementRef;

use super::node::traits::BorrowNode;

pub type NodeRef<Hash> = crate::arena::tl_element_ref::TLElementRef<Hash>;

impl<Hash> NodeRef<Hash>
where Hash: Clone + PartialEq + crate::hash::traits::Hashable
{
    fn hash<Node, Nodes, Hasher: crate::hash::traits::Hasher>(&self, nodes: &Nodes, hasher: &mut Hasher) 
    where Node: crate::tree::node::traits::Node<Hash=Hash>,
          Nodes: crate::tree::node::traits::Nodes<Node=Node>
    {
        
    }
}

impl<Hash> crate::hash::traits::Hashable for NodeRef<Hash>
where Hash: Clone + PartialEq + crate::hash::traits::Hashable
{
    fn hash<H: crate::hash::traits::Hasher>(&self, hasher: &mut H) {
       self.get_foreign_index().unwrap().hash(hasher)
    }
}