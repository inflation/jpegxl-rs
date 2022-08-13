# jpegxl-rs

[![Documentation](https://docs.rs/jpegxl-rs/badge.svg)](https://docs.rs/jpegxl-rs/)
[![Crates.io](https://img.shields.io/crates/v/jpegxl-rs.svg)](https://crates.io/crates/jpegxl-rs)
[![CI](https://github.com/inflation/jpegxl-rs/workflows/CI/badge.svg)](
https://github.com/inflation/jpegxl-rs/actions?query=workflow%3ACI)
[![Coverage Status](https://coveralls.io/repos/github/inflation/jpegxl-rs/badge.svg?branch=master)](
https://coveralls.io/github/inflation/jpegxl-rs?branch=master)
[![License: GPL-3.0-or-later](https://img.shields.io/crates/l/jpegxl-rs)](
https://github.com/inflation/jpegxl-rs/blob/master/LICENSE)

A safe JPEGXL wrapper over `libjxl` library. Check out the original [library](https://github.com/libjxl/libjxl)
and the [bindings](https://github.com/inflation/jpegxl-sys).

## Building

If you wish to specify a custom library path, set `DEP_JXL_LIB` environment variable.

Building `libjxl` and statically linking can be enabled by using `vendored` feature.
You can provide the source code with all third-party dependencies by `DEP_JXL_PATH`,
or it would fetch the code by `git`.

If you don't want to depend on C++ standard library, disable feature `threads`.

## Usage

Currently, `u8`, `u16` and `f32`(partial) are supported as pixel types. `u32` is in the header but not implemented.

Note: `f32` with alpha channel is not supported in encoder.

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

decoder.decode_to::<u8>(&sample);

// You can change the settings after initialization
decoder.num_channels = 1;
decoder.endianness = Endianness::Native;
```

### Encoding

```rust
use image::io::Reader as ImageReader;
let sample = ImageReader::open("../samples/sample.png")?.decode()?.to_rgba16();
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

let sample = std::fs::read("../samples/sample.jxl")?;
let decoder = decoder_builder().build()?;
let img = decoder.decode(&sample)?.into_dynamic_image();
```

License: GPL-3.0-or-later
