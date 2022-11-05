use sha2::Digest;

#[derive(Clone, PartialEq, Hash, Eq, Copy, Debug)]
pub struct BlockHash(pub [u8; 32]);

impl AsRef<[u8;32]> for BlockHash {
    fn as_ref<'a>(&'a self) -> &'a [u8;32] {
        &self.0
    }
}

impl AsRef<[u8]> for BlockHash {
    fn as_ref<'a>(&'a self) -> &'a [u8] {
        &self.0
    }
}

impl AsRef<BlockHash> for BlockHash {
    fn as_ref<'a>(&'a self) -> &'a Self {
        &self
    }
}

impl<const LENGTH: usize> From<&Block<LENGTH>> for BlockHash {
    fn from(block: &Block<LENGTH>) -> BlockHash {
        BlockHash(crate::hash::sha256(block))
    }
}

#[derive(PartialEq, Debug)]
pub struct Block<const LENGTH: usize>(pub [u8; LENGTH]);

impl<const LENGTH: usize> AsRef<[u8]> for Block<LENGTH> 
{
    fn as_ref<'a>(&'a self) -> &'a[u8]
    {
        return &self.0
    }
}

impl<const LENGTH: usize> AsRef<Self> for Block<LENGTH> 
{
    fn as_ref<'a>(&'a self) -> &Self
    {
        return &self
    }
}

impl<const LENGTH: usize> Block<LENGTH> {
    pub fn new() -> Self {
        Self([0; LENGTH])
    }

    pub fn hash<H: Digest>(&self, hasher: &mut H) {
        hasher.update(self.0);
    }
    
    pub fn can_hold(data: &[u8], offset: usize) -> bool
    {
        data.len() <= (LENGTH - offset)
    }

    pub fn append(&mut self, data: impl AsRef<[u8]>, offset: usize) -> bool {
        if Self::can_hold(data.as_ref(), offset) == false {
            return false;
        }

        self.0[offset..data.as_ref().len()].copy_from_slice(data.as_ref());
        return true;
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use rand::Rng;
    use sha2::Sha256;

    pub fn block_fixture<const LENGTH: usize>() -> Block<LENGTH>
    {
        let mut block = Block::<LENGTH>::new();
        let data = std::array::from_fn::<u8, LENGTH, _>(|_i| rand::thread_rng().gen()); 
        block.append(&data, 0);
        return block;            
    }

    #[test]
    fn test_block_append_success() 
    {
        let mut block = Block::<10>::new();
        let data = std::array::from_fn::<u8, 10, _>(|i| i.try_into().unwrap());
    
        assert_eq!(block.append(&data, 0), true);
        assert_eq!(block.0, data);

        let mut block_hasher = Sha256::new();
        let mut hasher = Sha256::new();
        hasher.update(block.0);
        block.hash(&mut block_hasher);

        assert_eq!(block_hasher.finalize(), hasher.finalize())
    }

    #[test]
    fn test_block_append_failure() 
    {
        let mut block = Block::<10>::new();
        let data = std::array::from_fn::<u8, 12, _>(|i| i.try_into().unwrap());
        assert_eq!(block.append(&data, 0), false);
    }
}