use std::ops::AddAssign;


#[derive(Ord, Eq, PartialOrd, PartialEq, Default, Clone, Hash, Copy)]
pub struct LocalIndex(pub i32);

impl AddAssign<i32> for LocalIndex {
    fn add_assign(&mut self, rhs: i32) {
        self.0 += rhs;
    }
}