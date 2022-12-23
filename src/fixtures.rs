use rand::rngs::OsRng;
use rand::RngCore;

use crate::io::{DataBuffer};

/// Create a random array of raw bytes.
pub fn random_raw_data(size: usize) -> DataBuffer {
    let mut data = DataBuffer::with_size(size);
    OsRng.fill_bytes(&mut data);
    data
}