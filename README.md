# jpegxl-rs

[![Documentation](https://docs.rs/jpegxl-rs/badge.svg)](https://docs.rs/jpegxl-rs/)
[![Crates.io](https://img.shields.io/crates/v/jpegxl-rs.svg)](https://crates.io/crates/jpegxl-rs)
[![CI](https://github.com/inflation/jpegxl-rs/workflows/CI/badge.svg)](
https://github.com/inflation/jpegxl-rs/actions?query=workflow%3ACI)
[![codecov](https://codecov.io/gh/inflation/jpegxl-rs/branch/master/graph/badge.svg?token=3WMRUQ816H)](
https://codecov.io/gh/inflation/jpegxl-rs)
[![License: GPL-3.0-or-later](https://img.shields.io/crates/l/jpegxl-rs)](
https://github.com/inflation/jpegxl-rs/blob/master/LICENSE)

A safe JPEGXL wrapper over `libjxl` library. Check out the original [library](https://github.com/libjxl/libjxl)
and the [bindings](https://github.com/inflation/jpegxl-rs/tree/master/jpegxl-sys).

## Building

If you wish to specify a custom library path, set the `DEP_JXL_LIB` environment variable.

Building `libjxl` and statically linking can be enabled by using the `vendored` feature.

If you don't want to depend on C++ standard library, disable the feature `threads`.

## Usage

Currently, `u8`, `u16`, `f16` and `f32` are supported as pixel types.

### Decoding

```rust
let mut decoder = decoder_builder().build()?;
let (Metadata { width, height, ..}, data) = decoder.pixels().decode(&sample)?;
match data {
    Data::Float(data) => { /* do something with Vec<f32> data */ },
    Data::Uint8(data) => { /* do something with Vec<u8> data */ },
    Data::Uint16(data) => { /* do something with Vec<u16> data */ },
    Data::Float16(data) => { /* do something with Vec<f16> data */ },
}

// Multi-threading
use jpegxl_rs::ThreadsRunner;
let runner = ThreadsRunner::default();
let mut decoder = decoder_builder()
                      .parallel_runner(&runner)
                      .build()?;

// Customize pixel format
let mut decoder = decoder_builder()
                      .pixel_format(PixelFormat {
                          num_channels: 3,
                          endianness: Endianness::Big,
                          align: 8
                      })
                      .build()?;

decoder.pixels().decode_to::<u8>(&sample);

// You can change the settings after initialization
decoder.pixel_format = Some(PixelFormat {
                                num_channels: 1,
                                endianness: Endianness::Native,
                                ..Default::default()
                            });

// Reconstruct JPEG
let (metadata, jpeg) = decoder.jpeg().reconstruct(&sample)?;
// Fallback to pixels if JPEG reconstruction fails
let (metadata, jpeg, pixels) = decoder.jpeg().reconstruct_with_pixels(&sample)?;

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

The integration is enabled by default. If you don't need it, disable `image` feature.

```rust
use jpegxl_rs::image::ToDynamic;
use jpegxl_rs::decoder_builder;
use image::DynamicImage;

let sample = std::fs::read("../samples/sample.jxl")?;
let mut decoder = decoder_builder().build()?;
let img = decoder.pixels().decode_to_dynamic_image(&sample)?;
```

License: GPL-3.0-or-later
