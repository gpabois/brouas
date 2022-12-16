/// Header of an overflow page
/// Size: 64 bytes
pub struct OverflowHeader
{
    /// 0 : No overflow, else the next overflow page
    next: u64
}
