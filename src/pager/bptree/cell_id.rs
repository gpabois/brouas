
#[derive(Clone, Copy)]
/// Represent a cell index
/// Useful to perform cell related operations (insert, remove, ...)
pub struct BPTreeCellId(u8);

impl std::ops::Add<i8> for BPTreeCellId {
    type Output = BPTreeCellId;

    fn add(self, rhs: i8) -> Self 
    {
        Self(self.0.wrapping_add_signed(rhs))
    }
}

impl std::ops::Sub<i8> for BPTreeCellId {
    type Output = BPTreeCellId;

    fn sub(self, rhs: i8) -> Self 
    {
        Self(self.0.wrapping_add_signed(-rhs))
    }
}

impl From<u8> for BPTreeCellId {
    fn from(value: u8) -> Self {
        Self(value)
    }
}


pub struct BPTreeCellIndexes(Vec<BPTreeCellId>);

impl BPTreeCellIndexes {
    pub fn new(len: u8) -> Self {
        (0u8..len).map(BPTreeCellId::from).collect()

    }

    pub fn iter(&self) -> impl Iterator<Item = &BPTreeCellId> {
        self.0.iter()
    }
}

impl FromIterator<BPTreeCellId> for BPTreeCellIndexes {
    fn from_iter<T: IntoIterator<Item = BPTreeCellId>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}


