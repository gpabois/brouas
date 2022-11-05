use crate::hash::sha256_string;
use crate::block::{Block, BlockHash};
use crate::merkle::MerkleNode;
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

impl<const BLOCK_LENGTH: usize> Storage<BlockHash, Block<BLOCK_LENGTH>> for LocalBlockStorage<BLOCK_LENGTH>
{
    fn save(&self, block: impl AsRef<Block<BLOCK_LENGTH>>) -> std::io::Result<()>
    {
        let fp = self.directory.join(sha256_string(block.as_ref()));
        std::fs::write(fp, block.as_ref())
    }

    fn load(&self, hash: impl AsRef<BlockHash>) -> std::io::Result<Block<BLOCK_LENGTH>>
    {
        let id = hex::encode(hash.as_ref());
        let fp = self.directory.join(id);
        let data = std::fs::read(fp)?;
        let mut block = Block::<BLOCK_LENGTH>::new();
        block.append(data, 0);
        Ok(block)
    }          
}

pub struct LocalMerkleNodeStorage<BlockStore: Storage<BlockHash, Block<65>>> {
    store: BlockStore
}

impl<BlockStore: Storage<BlockHash, Block<65>>> Storage<BlockHash, MerkleNode> for LocalMerkleNodeStorage<BlockStore>
{
    fn save(&self, node: impl AsRef<MerkleNode>) -> std::io::Result<()>
    {
        let mut block = Block::<65>::new();
        let data: [u8; 65] = node.as_ref().into();
        block.append(data, 0);
        self.store.save(block)?;
        Ok(())
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