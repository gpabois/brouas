use rand::rngs::OsRng;
use rand::RngCore;

use crate::{io::{DataBuffer}, pager::{page::PageSize, Pager}};

/// Create a random array of raw bytes.
pub fn random_raw_data(size: usize) -> DataBuffer {
    let mut data = DataBuffer::with_size(size);
    OsRng.fill_bytes(&mut data);
    data
}

pub fn pager_fixture(page_size: PageSize) -> Pager<DataBuffer> {
    Pager::new(DataBuffer::new(), page_size)
}