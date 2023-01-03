pub mod io;
pub mod buffer;
pub mod counter;
pub mod meta;
pub mod error;
pub mod result;

pub type ObjectId = u64;
pub type ObjectType = u16;
pub type ObjectSize = usize;

/// Reserved object types
pub const BPTREE_BRANCH: u16 = 0x0010;
pub const BPTREE_LEAF: u16   = 0x0011;


