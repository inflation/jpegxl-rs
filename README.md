# jpegxl-rs

[![Documentation](https://docs.rs/jpegxl-rs/badge.svg)](https://docs.rs/jpegxl-rs/)
[![Crates.io](https://img.shields.io/crates/v/jpegxl-rs.svg)](https://crates.io/crates/jpegxl-rs)
[![CI](https://github.com/inflation/jpegxl-rs/workflows/CI/badge.svg)](https://github.com/inflation/jpegxl-rs/actions?query=workflow%3ACI)
[![License: GPL-3.0-or-later](https://img.shields.io/crates/l/jpegxl-rs)](LICENSE)

`jpegxl-rs` is a safe wrapper over `jpeg-xl` library. Check out the original [library](https://gitlab.com/wg1/jpeg-xl)
and the [bindings](https://github.com/inflation/jpegxl-sys).

## Building

The library build `jpeg-xl` and link to `libc++` by default. Optionally, you can set `--features without-build`, then
set the include path and lib path with `DEP_JXL_INCLUDE` and `DEP_JXL_LIB` respectively.

If you don't want to depend on C++ standard library, use `--features without-threads` to disable default threadpool.
Instead, you can enable `with-rayon` feature to use its threadpool.

You need to have a working `llvm` environment.

## Usage

### Decoding

```rust
use jpegxl_rs::*;
let sample = std::fs::read("test/sample.jxl")?;
let mut decoder: JXLDecoder<u8> = decoder_builder().build()?;
let (info, buffer) = decoder.decode(&sample)?;
```

Set output pixel paramaters

```rust
// Pixel type is set by type parameter
let mut decoder: JXLDecoder<u16> = decoder_builder()
                                    .num_channel(3)
                                    .endianness(Endianness::Big)
                                    .align(8)
                                    .build()?;
```

### Encoding

```rust
use jpegxl_rs::*;
use image::io::Reader as ImageReader;

let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgba16();
let mut encoder = encoder_builder().build()?;
let buffer: Vec<u8> = encoder.encode(
                        &sample, 
                        sample.width() as _, 
                        sample.height() as _
                      )?;
```

### [`image`](https://crates.io/crates/image) crate integration

The integration is enabled by default. If you don't need it, use `without-image` feature.

```rust
use jpegxl_rs::image::*;
use image::DynamicImage;

let sample = std::fs::read("test/sample.jxl")?;
let decoder: JXLImageDecoder<u16> = JXLImageDecoder::new(&sample)?;
let img = DynamicImage::from_decoder(decoder)?;       
```
