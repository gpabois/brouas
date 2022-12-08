#[derive(Eq, PartialEq, Clone)]
pub struct StringIndex(String);

impl From<String> for StringIndex {
    fn from(value: String) -> Self {
        Self(value)
    }
}
 
impl crate::hash::traits::Hashable for StringIndex {
    fn hash<H: crate::hash::traits::Hasher>(&self, hasher: &mut H) {
        hasher.update(&self.0)
    }
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone)]
pub struct ByteIndex(u8);

impl From<u8> for ByteIndex {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl crate::hash::traits::Hashable for ByteIndex {
    fn hash<H: crate::hash::traits::Hasher>(&self, hasher: &mut H) {
        hasher.update(&[self.0])
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct BytesIndex(Vec<u8>);

impl crate::hash::traits::Hashable for BytesIndex {
    fn hash<H: crate::hash::traits::Hasher>(&self, hasher: &mut H) {
        hasher.update(&self.0)
    }
}