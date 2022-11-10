use ::image::io::Reader as ImageReader;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jpegxl_rs::*;
use parallel::threads_runner::ThreadsRunner;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Encoder");
    group.measurement_time(std::time::Duration::from_secs(120));

    let sample = ImageReader::open("../samples/bench.png")
        .unwrap()
        .decode()
        .unwrap()
        .to_rgb8();
    let mut encoder = encoder_builder().build().unwrap();
    group.bench_function("single thread", |b| {
        b.iter_with_large_drop(|| {
            encoder
                .encode::<_, u8>(black_box(sample.as_raw()), sample.width(), sample.height())
                .unwrap()
        })
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
        })
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
        })
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
