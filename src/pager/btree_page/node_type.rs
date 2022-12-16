pub enum BPTreeNodeType
{
    Leaf,
    Branch
}

impl Into<u8> for &BPTreeNodeType
{
    fn into(self) -> u8 {
        match self {
            BPTreeNodeType::Branch => 0,
            BPTreeNodeType::Leaf => 1
        }
    }
}

impl From<u8> for BPTreeNodeType
{
    fn from(value: u8) -> Self {
        match value {
            0 => BPTreeNodeType::Branch,
            1 => BPTreeNodeType::Leaf,
            _ => panic!("unknown type of b+ tree node")
        }
    }
}

