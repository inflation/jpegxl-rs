use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jpegxl_rs::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Decoder");
    group.measurement_time(std::time::Duration::new(30, 0));

    let sample = std::fs::read("test/sample.jxl").unwrap();
    let mut decoder = decoder_builder().build().unwrap();
    group.bench_function("single thread decode", |b| {
        b.iter_with_large_drop(|| decoder.decode::<u8>(black_box(&sample)))
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
