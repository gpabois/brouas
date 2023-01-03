#[derive(Debug)]
pub enum BPTreeError {
    BranchNotFound,
    LeafNotFound,
    ExistingKey,
    KeyNotFound
}
