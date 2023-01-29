/// A tree branch cell
/// Size: 128 bytes per cell
pub struct TreeBranchCell
{
    /// Pointer to the left child
    pub left_child: u64,
    /// The key
    pub index: u64,
}
