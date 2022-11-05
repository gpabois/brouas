use std::collections::HashMap;
use std::cell::RefCell;

use crate::hash::{sha256_string};
use crate::block::{Block, BlockHash};
use crate::merkle::{MerkleNode};
use super::Storage;

/// Stratégie I/O à base de systèmes de fichiers
pub struct LocalBlockStorage<const BLOCK_LENGTH: usize> {
    directory: std::path::PathBuf
}

impl<const BLOCK_LENGTH: usize> LocalBlockStorage<BLOCK_LENGTH>
{
    pub fn open(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            directory: path.into()
        }
    }
}

impl<const BLOCK_LENGTH: usize> Storage for LocalBlockStorage<BLOCK_LENGTH>
{
    type K = BlockHash;
    type V = Block<BLOCK_LENGTH>;

    fn flush(&self) -> std::io::Result<()>
    {
        Ok(())
    }

    fn exist(&self, hash: impl AsRef<Self::K>) -> bool
    {
        let id = hex::encode(hash.as_ref()); 
        let fp = self.directory.join(id);
        fp.exists()
    }

    fn save(&self, block: impl AsRef<Self::V>) -> std::io::Result<()>
    {
        let fp = self.directory.join(sha256_string(block.as_ref()));
        std::fs::write(fp, block.as_ref())
    }

    fn load(&self, hash: impl AsRef<Self::K>) -> std::io::Result<Self::V>
    {
        let id = hex::encode(hash.as_ref());
        let fp = self.directory.join(id);
        let data = std::fs::read(fp)?;
        let mut block = Block::<BLOCK_LENGTH>::new();
        block.append(data, 0);
        Ok(block)
    }          
}

pub type MerkleNodeBlock = Block<65>;

#[derive(Default)]
pub struct InMemoryMerkleNodeStorage {
    map: RefCell<HashMap<BlockHash, MerkleNode>>
}

impl Storage for InMemoryMerkleNodeStorage {
    type K = BlockHash;
    type V = MerkleNode;

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }

    fn exist(&self, k: impl AsRef<Self::K>) -> bool {
        self.map.borrow().contains_key(k.as_ref())
    }

    fn load(&self, k: impl AsRef<Self::K>) -> std::io::Result<Self::V> {
        match self.map.borrow().get(k.as_ref()) {
            Some(v) => Ok(v.clone()),
            None => Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Not found"))
        }
    }

    fn save(&self, v: impl AsRef<Self::V>) -> std::io::Result<()> {
        let k = BlockHash::from(v.as_ref());
        self.map.borrow_mut().insert(k, v.as_ref().clone());
        Ok(())
    }
}

pub struct LocalMerkleNodeStorage<BlockStore: Storage<K=BlockHash,V=MerkleNodeBlock>> {
    store: BlockStore
}

impl<BlockStore: Storage<K=BlockHash,V=MerkleNodeBlock>> Storage for LocalMerkleNodeStorage<BlockStore>
{
    type K = BlockHash;
    type V = MerkleNode;

    fn flush(&self) -> std::io::Result<()> {
        self.store.flush()
    }

    fn exist(&self, k: impl AsRef<Self::K>) -> bool {
        self.store.exist(k)    
    }

    fn save(&self, node: impl AsRef<MerkleNode>) -> std::io::Result<()>
    {
        let mut block = MerkleNodeBlock::new();
        let data: [u8; 65] = node.as_ref().into();
        block.append(data, 0);

        self.store.save(block)
    }

    fn load(&self, hash: impl AsRef<BlockHash>) -> std::io::Result<MerkleNode>
    {
        Ok(MerkleNode::from(self.store.load(hash)?.0))
    }  
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::block::tests::block_fixture;

    #[test]
    fn test_local_block_storage() -> std::io::Result<()> {
        let fp = std::path::Path::new("tests/blocks");
        std::fs::create_dir_all(&fp)?;

        let block = block_fixture::<10>();
        let block_hash = BlockHash::from(&block);

        let store = LocalBlockStorage::<10>::open(&fp);
        store.save(&block)?;    
        let stored_block = store.load(&block_hash)?;

        assert_eq!(block, stored_block);

        std::fs::remove_dir_all(&fp)?;
        Ok(())
    }

}