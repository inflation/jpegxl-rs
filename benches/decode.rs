use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jpegxl_rs::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    let sample = std::fs::read("test/sample_huge.jxl").unwrap();
    let mut decoder = JpegxlDecoder::new_with_default().unwrap();
    c.bench_function("singlethread decode", |b| {
        b.iter(|| decoder.decode(black_box(&sample)))
    });
}

pub fn criterion_benchmark_parallel(c: &mut Criterion) {
    let sample = std::fs::read("test/sample_huge.jxl").unwrap();
    let mut parallel_runner = ParallelRunner::new();
    let mut decoder = JpegxlDecoder::new(None, Some(&mut parallel_runner)).unwrap();
    c.bench_function("multithread decode", |b| {
        b.iter(|| decoder.decode(black_box(&sample)))
    });
}

criterion_group!(benches, criterion_benchmark, criterion_benchmark_parallel);
criterion_main!(benches);
