/// A tree branch cell
/// Size: 128 bytes per cell
pub struct TreeBranchCell
{
    /// Pointer to the left child
    left_child: u64,
    /// The key
    element_id: u64,
}
