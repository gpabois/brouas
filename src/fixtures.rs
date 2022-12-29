use rand::rngs::OsRng;
use rand::RngCore;
use rand::Rng;
use rand::seq::SliceRandom;

use crate::{io::{Data, InMemory}, pager::{traits::Pager as TraitPager, page::{result::PageResult, id::PageId, page_type::PageType}}, pager::{Pager, page::size::PageSize, storage::{PagerStream}}};

/// Create a random array of raw bytes.
pub fn random_data(size: usize) -> Data {
    let mut data = Data::with_size(size);
    OsRng.fill_bytes(&mut data);
    data
}

pub fn random_u64(min: u64, max: u64) -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..max)
}

pub fn random_page_type() -> PageType {
    let ls: Vec<u8> = (0u8..7u8).collect();
    ls.choose(&mut rand::thread_rng()).unwrap().clone().into()
}

pub fn random_page<P: TraitPager>(p: &mut P) -> PageResult<PageId> {
    let pg_id = p.new_page(random_page_type())?;
    p.write_all_to_page(&pg_id, &random_data(1000), 0usize)?;
    Ok(pg_id)
}

pub fn random_pages<P: TraitPager>(p: &mut P, nb: usize) -> PageResult<()> {
    let r: Result<Vec<_>, _> = (0..nb).map(|_| random_page(p)).collect();
    r?;
    Ok(())
}

pub fn pager_fixture(page_size: impl Into<PageSize>) -> Pager<PagerStream<InMemory>> {
    Pager::new(PagerStream::new(InMemory::new()), page_size.into())
}

pub fn pager_fixture_with_pages(page_size: impl Into<PageSize>, nb_pages: usize) -> Pager<PagerStream<InMemory>> {
    let mut pager = Pager::new(PagerStream::new(InMemory::new()), page_size.into());
    random_pages(&mut pager, nb_pages).unwrap();
    pager.flush().unwrap();
    pager.close_all().unwrap();
    pager
}