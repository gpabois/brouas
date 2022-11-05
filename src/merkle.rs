use crate::block::BlockHash;

#[derive(Clone)]
pub enum MerkleNode {
    Branch(BlockHash, BlockHash),
    Leaf(BlockHash)
}

impl From<&MerkleNode> for [u8; 65] {
    fn from(node: &MerkleNode) -> Self {
        let mut data: [u8; 65] = [0; 65];
        
        match node {
            MerkleNode::Branch(left_hash, right_hash) => {
                data[0] = 0x1;
                data[1..33].copy_from_slice(&left_hash.0);
                data[34..].copy_from_slice(&right_hash.0);
            },
            MerkleNode::Leaf(hash) => {
                data[1..33].copy_from_slice(&hash.0);
            }
        }

        return data;   
    }
}

impl From<[u8; 65]> for MerkleNode {
    fn from(data: [u8; 65]) -> Self {
        if data[0] > 0 {
            let mut hash_left: [u8; 32] = [0; 32];
            let mut hash_right: [u8; 32] = [0; 32];
            hash_left.copy_from_slice(&data[1..33]);
            hash_right.copy_from_slice(&data[34..]);
            return MerkleNode::Branch(BlockHash(hash_left), BlockHash(hash_right));
        } else {
            let mut hash: [u8; 32] = [0; 32];
            hash.copy_from_slice(&data[1..33]);
            return MerkleNode::Leaf(BlockHash(hash));
        }                
    }
}

pub struct MerkleTree<Store: crate::storage::Storage<BlockHash, MerkleNode>> {
    store: Store,
    root_hash: Option<BlockHash>
}

impl<Store: crate::storage::Storage<BlockHash, MerkleNode>> MerkleTree<Store>
{
    pub fn right_traverse(&self) -> std::io::Result<Vec<MerkleNode>> {
        let mut path: Vec<MerkleNode> = vec![];
        let mut stack: Vec<BlockHash> = vec![];
        
        if self.root_hash.clone().is_some() {
            stack.push(self.root_hash.clone().unwrap());
        }

        while stack.len() > 0 {
            let h = stack.pop().unwrap();
            
            let node = self.store.load(h)?;
            path.push(node.clone());

            match node {
                MerkleNode::Branch(_, right_hash) => stack.push(right_hash),
                _ => {}
            }
        }

        return Ok(path);
    }

}