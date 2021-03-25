use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jpegxl_rs::*;
use parallel::ThreadsRunner;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Decoder");
    group.measurement_time(std::time::Duration::new(20, 0));

    let sample = std::fs::read("test/sample.jxl").unwrap();
    let decoder = decoder_builder().build().unwrap();
    group.bench_function("single thread", |b| {
        b.iter_with_large_drop(|| decoder.decode::<u8>(black_box(&sample)).unwrap())
    });

    let parallel_runner = ThreadsRunner::default();
    let decoder = decoder_builder()
        .parallel_runner(&parallel_runner)
        .build()
        .unwrap();
    group.bench_function("c++ threadpool", |b| {
        b.iter_with_large_drop(|| decoder.decode::<u8>(black_box(&sample)).unwrap())
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
