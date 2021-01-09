use jpegxl_rs::threadpool::*;
use jpegxl_rs::*;

fn main() {
    let runner = Box::new(ThreadPoolRunner::new(1));
    let mut dec: JXLDecoder<u8> = decoder_builder().parallel_runner(runner).build();

    let sample = std::fs::read("test/sample.jxl").unwrap();
    dec.decode(&sample).unwrap();
}
