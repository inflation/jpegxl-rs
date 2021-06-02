# jpegxl-rs

[![Documentation](https://docs.rs/jpegxl-rs/badge.svg)](https://docs.rs/jpegxl-rs/)
[![Crates.io](https://img.shields.io/crates/v/jpegxl-rs.svg)](https://crates.io/crates/jpegxl-rs)
[![CI](https://github.com/inflation/jpegxl-rs/workflows/CI/badge.svg)](
https://github.com/inflation/jpegxl-rs/actions?query=workflow%3ACI)
[![Coverage Status](https://coveralls.io/repos/github/inflation/jpegxl-rs/badge.svg?branch=master)](
https://coveralls.io/github/inflation/jpegxl-rs?branch=master)
[![License: GPL-3.0-or-later](https://img.shields.io/crates/l/jpegxl-rs)](
https://github.com/inflation/jpegxl-rs/blob/master/LICENSE)

A safe JPEGXL wrapper over `jpeg-xl` library. Check out the original [library](https://gitlab.com/wg1/jpeg-xl)
and the [bindings](https://github.com/inflation/jpegxl-sys).

## Building

The library build `jpeg-xl` and link to `lib(std)c++` by default. Optionally, you can use `system-jxl` feature to
dynamically link to it. If you don't have it in the default include and library paths,
set them with `DEP_JXL_INCLUDE` and `DEP_JXL_LIB` respectively.

If you don't want to depend on C++ standard library, disable `threads` feature.

## Usage

### Decoding

```rust
let mut decoder = decoder_builder().build()?;

// Multi-threading
use jpegxl_rs::ThreadsRunner;
let runner = ThreadsRunner::default();
let mut decoder = decoder_builder()
                      .parallel_runner(&runner)
                      .build()?;

// Customize pixel format
let mut decoder = decoder_builder()
                      .num_channels(3)
                      .endianness(Endianness::Big)
                      .align(8)
                      .build()?;

// You can change the settings after initialization
decoder.num_channels = 1;
decoder.endianness = Endianness::Native;
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
encoder.lossless = false;
encoder.quality = 3.0;
```

### [`image`](https://crates.io/crates/image) crate integration

The integration is enabled by default. If you don't need it, disable `image-support` feature.

```rust
use jpegxl_rs::image::ToDynamic;
use jpegxl_rs::decoder_builder;
use image::DynamicImage;

let sample = std::fs::read("test/sample.jxl")?;
let decoder = decoder_builder().build()?;
let img = decoder.decode::<u8>(&sample)?.into_dynamic_image();
```

License: GPL-3.0-or-later
