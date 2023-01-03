use self::{node::BPTreeNodeId, nodes::traits::BPTreeNodes, result::BPTreeResult};

pub mod nodes;
pub mod node;
pub mod leaf;
pub mod branch;
pub mod result;
pub mod error;
pub mod alg;

pub struct BPTree(usize, Option<BPTreeNodeId>);

impl BPTree {
    pub fn new(capacity: usize) -> Self {
        Self(capacity, None)
    }

    pub fn get_capacity(&self) -> usize {
        self.0
    }
    
    pub fn set_root(&mut self, root: Option<BPTreeNodeId>) {
        self.1 = root;
    }

    pub fn get_root(&self)  -> Option<BPTreeNodeId> {
        self.1
    }

    pub fn insert<Nodes>(&mut self, nodes: &mut Nodes, key: Nodes::Key, value :Nodes::Value) -> BPTreeResult<()>
    where Nodes: BPTreeNodes
    {
        alg::insert(self, nodes, key, value)
    }

    pub fn contains<Nodes>(&self, nodes: &Nodes, key: &Nodes::Key) -> BPTreeResult<bool> 
    where Nodes: BPTreeNodes    
    {
        alg::contains(self, nodes, key)
    }
}

#[cfg(test)]
mod tests {
    use crate::{fixtures, io::Data};

    use super::{result::BPTreeResult, BPTree};


    #[test]
    pub fn test_bptree() -> BPTreeResult<()> {
        let mut nodes = fixtures::bptree::nodes_fixture::<usize, Data>();
        let mut tree = BPTree::new(10);

        for i in 0..1000usize {
            tree.insert(&mut nodes, i, fixtures::random_data(100))?;
        }
        
        assert!(tree.get_root().is_some());
        assert!(tree.contains(&nodes, &500usize)?);

        Ok(())
    }
}