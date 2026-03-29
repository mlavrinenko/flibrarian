use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use flibrarian_core::faking::fb2::generate_fb2_xml;
use flibrarian_core::indexing::{deserialize_fb2, parse_book_from_bytes};
use rand::SeedableRng;
use rand::rngs::StdRng;

fn generate_sample_fb2(seed: u64) -> String {
    let mut rng = StdRng::seed_from_u64(seed);
    generate_fb2_xml(&mut rng, "en")
}

fn bench_parse_book_from_bytes(c: &mut Criterion) {
    let samples: Vec<(u32, Vec<u8>)> = (0u32..10)
        .map(|i| {
            let xml = generate_sample_fb2(u64::from(42 + i));
            (i + 1, xml.into_bytes())
        })
        .collect();

    c.bench_function("parse_book_from_bytes", |b| {
        b.iter(|| {
            for (id, bytes) in &samples {
                let _ = parse_book_from_bytes(*id, bytes);
            }
        });
    });
}

fn bench_deserialize_fb2(c: &mut Criterion) {
    let samples: Vec<Vec<u8>> = (0u32..10)
        .map(|i| generate_sample_fb2(u64::from(42 + i)).into_bytes())
        .collect();

    c.bench_function("deserialize_fb2", |b| {
        b.iter(|| {
            for bytes in &samples {
                let _ = deserialize_fb2(bytes);
            }
        });
    });
}

fn bench_batch_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_parse");
    for size in [10u32, 50, 100] {
        let samples: Vec<(u32, Vec<u8>)> = (0..size)
            .map(|i| {
                let xml = generate_sample_fb2(u64::from(42 + i));
                (i + 1, xml.into_bytes())
            })
            .collect();

        group.bench_with_input(BenchmarkId::from_parameter(size), &samples, |b, samples| {
            b.iter(|| {
                for (id, bytes) in samples {
                    let _ = parse_book_from_bytes(*id, bytes);
                }
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_parse_book_from_bytes,
    bench_deserialize_fb2,
    bench_batch_sizes
);
criterion_main!(benches);
