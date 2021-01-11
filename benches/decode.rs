use criterion::{black_box, criterion_group, criterion_main, Criterion};
#[cfg(feature = "with-rayon")]
use jpegxl_rs::parallel::RayonRunner;
use jpegxl_rs::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Decoder");
    group.sample_size(10);

    let sample = std::fs::read("test/sample.jxl").unwrap();
    let mut decoder: JXLDecoder<u8> = decoder_builder().build();
    group.bench_function("c++ threadpool decode", |b| {
        b.iter(|| decoder.decode(black_box(&sample)))
    });

    #[cfg(feature = "with-rayon")]
    {
        let parallel_runner = Box::new(RayonRunner::new(Some(8)));
        decoder = decoder_builder().parallel_runner(parallel_runner).build();
        group.bench_function("rust threads decode", |b| {
            b.iter(|| decoder.decode(black_box(&sample)))
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
