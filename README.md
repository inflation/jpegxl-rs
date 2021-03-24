# jpegxl-rs

[![Documentation](https://docs.rs/jpegxl-rs/badge.svg)](https://docs.rs/jpegxl-rs/)
[![Crates.io](https://img.shields.io/crates/v/jpegxl-rs.svg)](https://crates.io/crates/jpegxl-rs)
[![CI](https://github.com/inflation/jpegxl-rs/workflows/CI/badge.svg)](https://github.com/inflation/jpegxl-rs/actions?query=workflow%3ACI)
[![License: GPL-3.0-or-later](https://img.shields.io/crates/l/jpegxl-rs)](https://github.com/inflation/jpegxl-rs/blob/master/LICENSE)

A safe JPEGXL wrapper over `jpeg-xl` library. Check out the original [library](https://gitlab.com/wg1/jpeg-xl)
and the [bindings](https://github.com/inflation/jpegxl-sys).

## Building

The library build `jpeg-xl` and link to `libc++` by default. Optionally, you can set `--features=system-jpegxl` to
dynamically link to it. If you don't have it in default include and library paths,
set them with `DEP_JXL_INCLUDE` and `DEP_JXL_LIB` respectively.

If you don't want to depend on C++ standard library, use `--features without-threads` to disable default threadpool.

## Usage

### Decoding

```rust
let mut decoder = decoder_builder().build()?;

// Use multithread
use jpegxl_rs::ThreadsRunner;
let runner = ThreadsRunner::default();
let mut decoder = decoder_builder()
                      .parallel_runner(&runner)
                      .build()?;

// Customize pixel format
let mut decoder = decoder_builder()
                      .num_channels(3)
                      .endian(Endianness::Big)
                      .align(8)
                      .build()?;

// Set custom memory manager
use jpegxl_rs::memory::MallocManager;
let manager = MallocManager::default();
let mut decoder = decoder_builder()
                      .memory_manager(&manager)
                      .build()?;
```

### Encoding

```rust
use image::io::Reader as ImageReader;
let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgba16();
let mut encoder = encoder_builder().build()?;
let buffer: EncoderResult<f32> = encoder.encode(&sample, sample.width(), sample.height())?;
```

Set encoder options

```rust
let mut encoder = encoder_builder()
                    .lossless(true)
                    .speed(EncoderSpeed::Falcon)
                    .build()?;
// You can change the settings after initialization
encoder.set_lossless(false);
encoder.set_quality(3.0);
```

### [`image`](https://crates.io/crates/image) crate integration

The integration is enabled by default. If you don't need it, use `without-image` feature.

```rust
use jpegxl_rs::image::*;
use image::DynamicImage;

let sample = std::fs::read("test/sample.jxl")?;
let decoder: JxlImageDecoder<u16> = JxlImageDecoder::new(&sample)?;
let img = DynamicImage::from_decoder(decoder)?;
```

License: GPL-3.0-or-later
