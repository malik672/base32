use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use base32::{encode, decode};

fn benchmark_decode(c: &mut Criterion) {
    let test_data = vec![
        ("small", b"hello".to_vec()),
        ("medium", b"hello world! this is a test".to_vec()),
        ("large", vec![0x42; 1000]),
        ("very_large", vec![0xAB; 10000]),
    ];

    let mut group = c.benchmark_group("decode");
    group.sample_size(1000);

    for (name, data) in test_data {
        // Pre-encode the data for decoding (returns Vec<u8>)
        let encoded_vec = encode(&data);
        // Convert to string for decode function
        let encoded_str = String::from_utf8(encoded_vec).unwrap();

        group.bench_with_input(BenchmarkId::new("decode", name), &encoded_str, |b, encoded| {
            b.iter(|| decode(black_box(encoded)));
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_decode);
criterion_main!(benches);
