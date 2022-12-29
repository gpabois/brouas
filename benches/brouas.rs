#[macro_use]
extern crate bencher;

use bencher::Bencher;
use brouas::{{pager::page::id::PageId, pager::TraitPager}, fixtures, pager::overflow::Overflow};

fn bench_pager_random_write_to_page(bench: &mut Bencher) {
    let nb_pages = 10000usize;
    let mut pager = fixtures::pager_fixture_with_pages(4000u64, nb_pages);
    let data = fixtures::random_data(3000);

    bench.iter(|| {
        let pg_id: PageId = fixtures::random_u64(1, nb_pages as u64).into();
        pager.open_page(&pg_id).unwrap();
        pager.write_to_page(&pg_id, &data, 0u32).unwrap();
        pager.close_page(&pg_id).unwrap();
    })
}

fn bench_pager_overflow_random_write(bench: &mut Bencher) {
    let nb_pages = 10000usize;
    let mut pager = fixtures::pager_fixture_with_pages(4000u64, nb_pages);
    let data = fixtures::random_data(1_000_000usize); 

    bench.iter(|| {
        Overflow::write(&mut pager, &mut data.get_cursor_read(), None).unwrap();
    })
}

benchmark_group!(pager_benches, bench_pager_random_write_to_page, bench_pager_overflow_random_write);
benchmark_main!(pager_benches);