use super::error::BPTreeError;

pub type BPTreeResult<T> = std::result::Result<T, BPTreeError>;
