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
let (Metadata { width, height, ..}, pixels) = decoder.decode(&sample)?;
match pixels {
    Pixels::Float(data) => { /* do something with Vec<f32> data */ },
    Pixels::Uint8(data) => { /* do something with Vec<u8> data */ },
    Pixels::Uint16(data) => { /* do something with Vec<u16> data */ },
    Pixels::Float16(data) => { /* do something with Vec<f16> data */ },
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

decoder.decode_to::<u8>(&sample);

// You can change the settings after initialization
decoder.skip_reorientation = Some(true);

// Reconstruct JPEG, fallback to pixels if JPEG reconstruction is not possible
// This operation is finished in on pass
let (metadata, data) = decoder.reconstruct(&sample)?;
match data {
    Data::Jpeg(jpeg) => {/* do something with the JPEG data */}
    Data::Pixels(pixels) => {/* do something with the pixels data */}
}

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
let img = decoder.decode_to_dynamic_image(&sample)?;
```

License: GPL-3.0-or-later
