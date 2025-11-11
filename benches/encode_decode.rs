use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use base32::{encode_into, encoded_len};

fn benchmark_encode_decode(c: &mut Criterion) {
    let test_data = vec![
        ("small", b"hello".to_vec()),
        ("medium", b"hello world! this is a test".to_vec()),
        ("large", vec![0x42; 1000]),
        ("very_large", vec![0xAB; 10000]),
    ];

    let mut group = c.benchmark_group("encode_decode");
    group.sample_size(1000);

    for (name, data) in test_data {
        // group.bench_with_input(BenchmarkId::new("encode", name), &data, |b, data| {
        //     b.iter(|| encode(black_box(data)));
        // });

     
        group.bench_with_input(
            BenchmarkId::new("encode_into", name),
            &data,
            |b, data| {
                let mut out = vec![0u8; encoded_len(data.len()) + 8];
                b.iter(|| {
                    encode_into(&mut out, black_box(data));
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_encode_decode);
criterion_main!(benches);
