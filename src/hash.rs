use sha2::Digest;

pub mod traits {
   
    pub trait Hasher
    {
        type Hash;
    
        fn update(&mut self, data: impl AsRef<[u8]>);
        fn finalize(self) -> Self::Hash;
    }

    pub trait Hash {
        type Hasher: Hasher<Hash=Self>;

        fn new_hasher() -> Self::Hasher;
    }
    
    pub trait Hashable
    {
        fn hash<H: Hasher>(&self, hasher: &mut H);
    }
}

pub struct Sha256Hasher {
    hasher: sha2::Sha256
}

impl Sha256Hasher {
    pub fn new() -> Self {
        Self {
            hasher: sha2::Sha256::new()
        }
    }
}

impl self::traits::Hasher for Sha256Hasher
{
    type Hash = Sha256;

    fn update(&mut self, data: impl AsRef<[u8]>) {
        self.hasher.update(data)
    }

    fn finalize(self) -> Self::Hash {
        let hash: Vec<u8> = self.hasher.finalize().into_iter().collect();
        Sha256 {
            data: hash
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct Sha256
{
    data: Vec<u8>
}

impl std::fmt::Display for Sha256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self.data)
    }
}

impl self::traits::Hash for Sha256 {
    type Hasher = Sha256Hasher;

    fn new_hasher() -> Self::Hasher {
        Sha256Hasher::new()
    }
}

impl self::traits::Hashable for Sha256 {
    fn hash<H: traits::Hasher>(&self, hasher: &mut H) {
        hasher.update(&self.data);
    }
}