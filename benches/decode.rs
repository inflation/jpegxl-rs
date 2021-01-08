use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jpegxl_rs::parallel::ParallelRunner;
use jpegxl_rs::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    let sample = std::fs::read("test/sample.jxl").unwrap();
    let mut decoder: JXLDecoder<u8> = decoder_builder().build();
    c.bench_function("c++ threadpool decode", |b| {
        b.iter(|| decoder.decode(black_box(&sample)))
    });
}

pub fn criterion_benchmark_rust(c: &mut Criterion) {
    let sample = std::fs::read("test/sample.jxl").unwrap();
    let parallel_runner = Box::new(ParallelRunner::default());
    let mut decoder: JXLDecoder<u8> = decoder_builder().parallel_runner(parallel_runner).build();
    c.bench_function("rust threads decode", |b| {
        b.iter(|| decoder.decode(black_box(&sample)))
    });
}

criterion_group!(benches, criterion_benchmark, criterion_benchmark_rust,);

criterion_main!(benches);
