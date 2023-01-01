
use num_bigint::{BigUint, BigInt};

pub enum Index {
    BigInt(BigInt),
    BigUint(BigUint)
}

impl PartialEq for Index {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::BigInt(l0), Self::BigInt(r0)) => l0 == r0,
            (Self::BigUint(l0), Self::BigUint(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl PartialOrd for Index {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::BigInt(l0), Self::BigInt(r0)) => l0.partial_cmp(r0),
            (Self::BigUint(l0), Self::BigUint(r0)) => l0.partial_cmp(r0),
            _ => None
        }
    }
}
