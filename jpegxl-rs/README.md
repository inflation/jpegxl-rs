# `jpegxl-rs`

[![Documentation](https://docs.rs/jpegxl-rs/badge.svg)][1]
[![Crates.io](https://img.shields.io/crates/v/jpegxl-rs.svg)][2]
[![dependency status](https://deps.rs/repo/github/inflation/jpegxl-rs/status.svg)][3]
[![CI](https://github.com/inflation/jpegxl-rs/workflows/CI/badge.svg)][4]
[![codecov](https://codecov.io/gh/inflation/jpegxl-rs/branch/master/graph/badge.svg?token=3WMRUQ816H)][5]
[![License: GPL-3.0-or-later](https://img.shields.io/crates/l/jpegxl-rs)][6]

[1]: https://docs.rs/jpegxl-rs/
[2]: https://crates.io/crates/jpegxl-rs
[3]: https://deps.rs/repo/github/inflation/jpegxl-rs
[4]: https://github.com/inflation/jpegxl-rs/actions?query=workflow%3ACI
[5]: https://codecov.io/gh/inflation/jpegxl-rs
[6]: https://github.com/inflation/jpegxl-rs/blob/master/LICENSE

A safe JPEGXL wrapper over `libjxl` library. Check out the original [library](https://github.com/libjxl/libjxl)
and the [bindings](https://github.com/inflation/jpegxl-rs/tree/master/jpegxl-sys).

## Building

If you wish to specify a custom library path, set the `DEP_JXL_LIB` environment variable.

To build `libjxl` and statically link it, use the `vendored` feature.

## Usage

Currently, you can use `u8`, `u16`, `f16`, and `f32` as pixel types.

### Decoding

```rust
use jpegxl_rs::*;
use jpegxl_rs::decode::*;

let mut decoder = decoder_builder().build().unwrap();
let sample = include_bytes!("../../samples/sample.jxl");

let (Metadata { width, height, ..}, pixels) = decoder.decode(sample).unwrap();
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
                      .build()
                      .unwrap();

// Customize pixel format
let mut decoder = decoder_builder()
                      .pixel_format(PixelFormat {
                          num_channels: 3,
                          endianness: Endianness::Big,
                          align: 8
                      })
                      .build()
                      .unwrap();

decoder.decode_with::<u8>(sample);

// You can change the settings after initialization
decoder.skip_reorientation = Some(true);

// Reconstruct JPEG, fallback to pixels if JPEG reconstruction is not possible
// This operation is finished in on pass
let (metadata, data) = decoder.reconstruct(sample).unwrap();
match data {
    Data::Jpeg(jpeg) => {/* do something with the JPEG data */}
    Data::Pixels(pixels) => {/* do something with the pixels data */}
}
```

### Encoding

```rust
use image::ImageReader;
use jpegxl_rs::encoder_builder;
use jpegxl_rs::encode::{EncoderResult, EncoderSpeed};

let sample = ImageReader::open("../samples/sample.png").unwrap().decode().unwrap().to_rgba16();
let mut encoder = encoder_builder().build().unwrap();

let buffer: EncoderResult<f32> = encoder.encode(&sample, sample.width(), sample.height()).unwrap();

// Set encoder options
let mut encoder = encoder_builder()
                    .lossless(true)
                    .speed(EncoderSpeed::Falcon)
                    .build()
                    .unwrap();

// You can change the settings after initialization
encoder.lossless = Some(false);
encoder.quality = 3.0;
```

### [`image`](https://crates.io/crates/image) crate integration

By default, this integration uses the `image` integration.
If you don't need it, turn off the `image` feature.

```rust
use jpegxl_rs::image::ToDynamic;
use jpegxl_rs::decoder_builder;
use image::DynamicImage;

let sample = std::fs::read("../samples/sample.jxl").unwrap();
let mut decoder = decoder_builder().build().unwrap();
let img = decoder.decode_to_image(&sample).unwrap();
let img = decoder.decode_to_image_with::<f32>(&sample).unwrap();
```

## MSRV

Following the latest stable Debian rustc version.
Currently: 1.85 (Debian Trixie)

License: `GPL-3.0-or-later`
