use rand::rngs::OsRng;
use rand::RngCore;
use rand::Rng;

use crate::io::Data;

//pub mod bptree;

/// Create a random array of raw bytes.
pub fn random_data(size: usize) -> Data {
    let mut data = Data::with_size(size);
    OsRng.fill_bytes(&mut data);
    data
}

pub fn randomise(data: &mut [u8]) {
    OsRng.fill_bytes(data);
}

pub fn random_u64(min: u64, max: u64) -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..max)
}

