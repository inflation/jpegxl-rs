use criterion::{black_box, criterion_group, criterion_main, Criterion};
#[cfg(feature = "with-rayon")]
use jpegxl_rs::parallel::RayonRunner;
use jpegxl_rs::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Decoder");
    group.sample_size(10);

    let sample = std::fs::read("test/sample.jxl").unwrap();
    let mut decoder: JxlDecoder<u8> = decoder_builder().build().unwrap();
    group.bench_function("c++ threadpool decode", |b| {
        b.iter(|| decoder.decode(black_box(&sample)))
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
