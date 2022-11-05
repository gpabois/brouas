use crate::block::{BlockHash};
use crate::storage::Storage;
use crate::storage::impls::InMemoryMerkleNodeStorage;

#[derive(Clone, PartialEq, Debug)]
pub enum MerkleNode {
    Branch(BlockHash, BlockHash),
    Leaf(BlockHash)
}

pub type MerkleNodeBytes = [u8;65];

impl From<&MerkleNode> for MerkleNodeBytes {
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

impl From<[u8; 65]> for MerkleNode 
{
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

impl AsRef<MerkleNode> for MerkleNode {
    fn as_ref<'a>(&'a self) -> &'a Self {
        self
    }
}

impl From<&MerkleNode> for BlockHash {
    fn from(node: &MerkleNode) -> BlockHash {
        BlockHash(crate::hash::sha256(<[u8; 65]>::from(node)))
    }
}

pub struct MerkleTree<Store: Storage<K=BlockHash,V=MerkleNode>> {
    store: Store,
    root_hash: Option<BlockHash>
}

impl MerkleTree<InMemoryMerkleNodeStorage>
{
    pub fn in_memory() -> Self {
        Self {
            store: InMemoryMerkleNodeStorage::default(),
            root_hash: None
        }
    }
}

impl<Store: Storage<K=BlockHash,V=MerkleNode>> MerkleTree<Store>
{
    /// Insère un nouveau hash dans l'arbre
    pub fn insert(&mut self, hash: impl Into<BlockHash>) -> std::io::Result<()> {
        let path = self.right_traverse()?;

        let leaf = MerkleNode::Leaf(hash.into());

        // On sauvegarde la feuille
        self.store.save(&leaf)?;
        
        // Premier cas: arbre vide, facile !
        if path.len() == 0 {
            self.root_hash = Some(BlockHash::from(&leaf));
        }

        Ok(())
    }

    pub fn root(&self) -> std::io::Result<Option<MerkleNode>>
    {
        if self.root_hash.is_none() {
            return Ok(None);
        }

        Ok(Some(self.store.load(self.root_hash.unwrap())?))
    }
    
    /// Récupère le chemin vers le bloc le plus à droite de l'arbre Merkle.
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

#[cfg(test)]
pub mod tests {
    use crate::block::tests::block_fixture;

    use super::*;

    #[test]
    fn test_merkle_tree() -> std::io::Result<()>
    {
        let mut tree = MerkleTree::in_memory();
        let hash = BlockHash::from(&block_fixture::<10>());
        tree.insert(hash)?;

        let root = tree.root()?.unwrap();

        assert_eq!(root, MerkleNode::Leaf(hash));

        Ok(())
    }
}