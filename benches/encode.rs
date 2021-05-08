use ::image::io::Reader as ImageReader;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jpegxl_rs::*;
use parallel::ThreadsRunner;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Encoder");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::new(15, 0));

    let sample = ImageReader::open("test/bench.png")
        .unwrap()
        .decode()
        .unwrap()
        .to_rgb8();
    let encoder = encoder_builder().build().unwrap();
    group.bench_function("single thread", |b| {
        b.iter_with_large_drop(|| {
            encoder
                .encode::<_, u8>(black_box(sample.as_raw()), sample.width(), sample.height())
                .unwrap()
        })
    });

    let parallel_runner = ThreadsRunner::default();
    let encoder = encoder_builder()
        .parallel_runner(&parallel_runner)
        .build()
        .unwrap();
    group.bench_function("c++ threadpool", |b| {
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
