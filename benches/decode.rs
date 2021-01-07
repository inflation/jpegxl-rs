use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jpegxl_rs::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    let sample = std::fs::read("test/sample.jxl").unwrap();
    c.bench_function("singlethread decode", |b| {
        b.iter(|| {
            let mut decoder: JxlDecoder<u8> = JxlDecoder::default().unwrap();
            decoder.decode(black_box(&sample))
        })
    });
}

pub fn criterion_benchmark_parallel(c: &mut Criterion) {
    let sample = std::fs::read("test/sample.jxl").unwrap();
    let mut parallel_runner = ParallelRunner::new();
    c.bench_function("multithread decode", |b| {
        b.iter(|| {
            let mut decoder: JxlDecoder<u8> =
                JxlDecoder::new(None, Some(&mut parallel_runner)).unwrap();
            decoder.decode(black_box(&sample))
        })
    });
}

#[cfg(not(feature = "without-threads"))]
pub fn criterion_benchmark_threads(c: &mut Criterion) {
    let sample = std::fs::read("test/sample.jxl").unwrap();
    let mut parallel_runner = ParallelRunner::new();
    c.bench_function("multithread decode", |b| {
        b.iter(|| {
            let mut decoder: JxlDecoder<u8> =
                JxlDecoder::new(None, Some(&mut parallel_runner)).unwrap();
            decoder.decode(black_box(&sample))
        })
    });
}

criterion_group!(benches, criterion_benchmark, criterion_benchmark_parallel,);
#[cfg(not(feature = "without-threads"))]
criterion_group!(benches_theads, criterion_benchmark_threads);

criterion_main!(benches, benches_theads);
