#![allow(missing_docs)]
#![allow(clippy::missing_panics_doc)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jpegxl_rs::{encoder_builder, ResizableRunner, ThreadsRunner};

const SAMPLE: &[u8] = include_bytes!("../../samples/bench.png");

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Encoder");
    group.measurement_time(std::time::Duration::from_secs(120));

    let sample = image::load_from_memory_with_format(SAMPLE, image::ImageFormat::Png)
        .unwrap()
        .to_rgb8();
    let mut encoder = encoder_builder().build().unwrap();
    group.bench_function("single thread", |b| {
        b.iter_with_large_drop(|| {
            encoder
                .encode::<_, u8>(black_box(sample.as_raw()), sample.width(), sample.height())
                .unwrap()
        });
    });

    let parallel_runner = ThreadsRunner::default();
    let mut encoder = encoder_builder()
        .parallel_runner(&parallel_runner)
        .build()
        .unwrap();
    group.bench_function("thread pool using default number of threads", |b| {
        b.iter_with_large_drop(|| {
            encoder
                .encode::<_, u8>(black_box(sample.as_raw()), sample.width(), sample.height())
                .unwrap()
        });
    });

    let parallel_runner = ResizableRunner::default();
    let mut encoder = encoder_builder()
        .parallel_runner(&parallel_runner)
        .build()
        .unwrap();
    group.bench_function("resizable thread pool", |b| {
        b.iter_with_large_drop(|| {
            encoder
                .encode::<_, u8>(black_box(sample.as_raw()), sample.width(), sample.height())
                .unwrap()
        });
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
