use flibrarian_core::faking::fb2::generate_fb2_xml;
use flibrarian_core::indexing::parse_book_from_bytes;
use iai_callgrind::{
    EventKind, LibraryBenchmarkConfig, RegressionConfig, library_benchmark,
    library_benchmark_group, main,
};
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::hint::black_box;

fn generate_sample_fb2(seed: u64) -> Vec<u8> {
    let mut rng = StdRng::seed_from_u64(seed);
    generate_fb2_xml(&mut rng, "en").into_bytes()
}

#[library_benchmark]
fn parse_single_book() {
    let bytes = generate_sample_fb2(42);
    let _ = black_box(parse_book_from_bytes(1, &bytes));
}

#[library_benchmark]
fn parse_batch() {
    for i in 0u32..10 {
        let bytes = generate_sample_fb2(u64::from(42 + i));
        let _ = black_box(parse_book_from_bytes(i + 1, &bytes));
    }
}

library_benchmark_group!(
    name = indexing_group;
    benchmarks = parse_single_book, parse_batch
);

main!(
    config = LibraryBenchmarkConfig::default().regression(
        RegressionConfig::default()
            .limits([(EventKind::Ir, 5.0), (EventKind::EstimatedCycles, 10.0)])
    );
    library_benchmark_groups = indexing_group
);
