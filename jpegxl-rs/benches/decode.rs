#![allow(missing_docs)]
#![allow(clippy::missing_panics_doc)]

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use jpegxl_rs::{decoder_builder, parallel, ResizableRunner};
use parallel::threads_runner::ThreadsRunner;

const SAMPLE: &[u8] = include_bytes!("../../samples/bench.jxl");

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Decoder");
    group.measurement_time(std::time::Duration::from_secs(120));

    let decoder = decoder_builder().build().unwrap();
    group.bench_function("single thread", |b| {
        b.iter_with_large_drop(|| decoder.decode_with::<u8>(black_box(SAMPLE)).unwrap());
    });

    let parallel_runner = ThreadsRunner::default();
    let decoder = decoder_builder()
        .parallel_runner(&parallel_runner)
        .build()
        .unwrap();
    group.bench_function("thread pool using default number of threads", |b| {
        b.iter_with_large_drop(|| decoder.decode_with::<u8>(black_box(SAMPLE)).unwrap());
    });

    let parallel_runner = ResizableRunner::default();
    let decoder = decoder_builder()
        .parallel_runner(&parallel_runner)
        .build()
        .unwrap();
    group.bench_function("resizable thread pool", |b| {
        b.iter_with_large_drop(|| decoder.decode_with::<u8>(black_box(SAMPLE)).unwrap());
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
