use rand::rngs::OsRng;
use rand::RngCore;

use crate::{io::{DataBuffer, InMemory}, pager::{Pager, page::size::PageSize, storage::{PagerStream}}};

/// Create a random array of raw bytes.
pub fn random_raw_data(size: usize) -> DataBuffer {
    let mut data = DataBuffer::with_size(size);
    OsRng.fill_bytes(&mut data);
    data
}

pub fn pager_fixture(page_size: impl Into<PageSize>) -> Pager<PagerStream<InMemory>> {
    Pager::new(PagerStream::new(InMemory::new()), page_size.into())
}