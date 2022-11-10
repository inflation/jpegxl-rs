use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jpegxl_rs::*;
use parallel::threads_runner::ThreadsRunner;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Decoder");
    group.measurement_time(std::time::Duration::from_secs(120));

    let sample = std::fs::read("../samples/bench.jxl").unwrap();
    let decoder = decoder_builder().build().unwrap();
    group.bench_function("single thread", |b| {
        b.iter_with_large_drop(|| decoder.decode_to::<u8>(black_box(&sample)).unwrap())
    });

    let parallel_runner = ThreadsRunner::default();
    let decoder = decoder_builder()
        .parallel_runner(&parallel_runner)
        .build()
        .unwrap();
    group.bench_function("thread pool using default number of threads", |b| {
        b.iter_with_large_drop(|| decoder.decode_to::<u8>(black_box(&sample)).unwrap())
    });

    let parallel_runner = ResizableRunner::default();
    let decoder = decoder_builder()
        .parallel_runner(&parallel_runner)
        .build()
        .unwrap();
    group.bench_function("resizable thread pool", |b| {
        b.iter_with_large_drop(|| decoder.decode_to::<u8>(black_box(&sample)).unwrap())
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
