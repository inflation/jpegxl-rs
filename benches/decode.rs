use criterion::{black_box, criterion_group, criterion_main, Criterion, SamplingMode};
use jpegxl_rs::*;
use parallel::ThreadsRunner;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Decoder");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(10);
    group.measurement_time(std::time::Duration::new(15, 0));

    let sample = std::fs::read("samples/bench.jxl").unwrap();
    let decoder = decoder_builder().build().unwrap();
    group.bench_function("single thread", |b| {
        b.iter_with_large_drop(|| decoder.decode_to::<u8>(black_box(&sample)).unwrap())
    });

    let parallel_runner = ThreadsRunner::default();
    let decoder = decoder_builder()
        .parallel_runner(&parallel_runner)
        .build()
        .unwrap();
    group.bench_function("c++ threadpool", |b| {
        b.iter_with_large_drop(|| decoder.decode_to::<u8>(black_box(&sample)).unwrap())
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
