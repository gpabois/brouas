use self::node::BPTreeNodeId;

pub mod nodes;
pub mod node;
pub mod leaf;
pub mod branch;
pub mod result;
pub mod error;
pub mod alg;

pub struct BPTree(usize, Option<BPTreeNodeId>);

impl BPTree {
    pub fn get_capacity(&self) -> usize {
        self.0
    }
    
    pub fn set_root(&mut self, root: Option<BPTreeNodeId>) {
        self.1 = root;
    }

    pub fn get_root(&self)  -> Option<BPTreeNodeId> {
        self.1
    }
}