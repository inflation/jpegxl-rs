/*
This file is part of jpegxl-rs.

jpegxl-rs is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

jpegxl-rs is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with jpegxl-rs.  If not, see <https://www.gnu.org/licenses/>.
*/

#![warn(missing_docs)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
// #![allow(clippy::enum_glob_use)]
// #![allow(clippy::default_trait_access)]

//! [![Documentation](https://docs.rs/jpegxl-rs/badge.svg)](https://docs.rs/jpegxl-rs/)
//! [![Crates.io](https://img.shields.io/crates/v/jpegxl-rs.svg)](https://crates.io/crates/jpegxl-rs)
//! [![CI](https://github.com/inflation/jpegxl-rs/workflows/CI/badge.svg)](
//! https://github.com/inflation/jpegxl-rs/actions?query=workflow%3ACI)
//! [![Coverage Status](https://coveralls.io/repos/github/inflation/jpegxl-rs/badge.svg?branch=master)](
//! https://coveralls.io/github/inflation/jpegxl-rs?branch=master)
//! [![License: GPL-3.0-or-later](https://img.shields.io/crates/l/jpegxl-rs)](
//! https://github.com/inflation/jpegxl-rs/blob/master/LICENSE)
//!
//! A safe JPEGXL wrapper over `libjxl` library. Check out the original [library](https://github.com/libjxl/libjxl)
//! and the [bindings](https://github.com/inflation/jpegxl-sys).
//!
//! # Building
//!
//! The library build `libjxl` and link to `libc++/libstdc++` by default.
//! Optionally, you can use `system-jxl` feature to dynamically link to it.
//! If you don't have it in the default include and library paths,
//! set them with `DEP_JXL_INCLUDE` and `DEP_JXL_LIB` respectively.
//!
//! If you don't want to depend on C++ standard library, disable feature `threads`.
//!
//! # Usage
//!
//! Currently, `u8`, `u16` and `f32`(partial) are supported as pixel types. `u32` is in the header but not implemented.
//!
//! Note: `f32` with alpha channel is not supported in encoder.
//!
//! ## Decoding
//!
//! ```
//! # use jpegxl_rs::*;
//! # || -> Result<(), Box<dyn std::error::Error>> {
//! # let sample = [];
//! let mut decoder = decoder_builder().build()?;
//!
//! // Multi-threading
//! use jpegxl_rs::ThreadsRunner;
//! let runner = ThreadsRunner::default();
//! let mut decoder = decoder_builder()
//!                       .parallel_runner(&runner)
//!                       .build()?;
//!
//! // Customize pixel format
//! let mut decoder = decoder_builder()
//!                       .num_channels(3)
//!                       .endianness(Endianness::Big)
//!                       .align(8)
//!                       .build()?;
//!
//! decoder.decode_to::<u8>(&sample);
//!
//! // You can change the settings after initialization
//! decoder.num_channels = 1;
//! decoder.endianness = Endianness::Native;
//! # Ok(()) };
//! ```
//!
//! ## Encoding
//!
//! ```
//! # use jpegxl_rs::{encoder_builder, encode::EncoderResult};
//! # || -> Result<(), Box<dyn std::error::Error>> {
//! use image::io::Reader as ImageReader;
//! let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgba16();
//! let mut encoder = encoder_builder().build()?;
//! let buffer: EncoderResult<f32> = encoder.encode(&sample, sample.width(), sample.height())?;
//! # Ok(()) };
//! ```
//!
//! Set encoder options
//!
//! ```
//! # || -> Result<(), Box<dyn std::error::Error>> {
//! # use jpegxl_rs::{*, encode::EncoderSpeed};
//! let mut encoder = encoder_builder()
//!                     .lossless(true)
//!                     .speed(EncoderSpeed::Falcon)
//!                     .build()?;
//! // You can change the settings after initialization
//! encoder.lossless = false;
//! encoder.quality = 3.0;
//! # Ok(()) };
//! ```
//!
//! ## [`image`](https://crates.io/crates/image) crate integration
//!
//! The integration is enabled by default. If you don't need it, disable `image-support` feature.
//!
//! ```
//! # || -> Result<(), Box<dyn std::error::Error>> {
//! use jpegxl_rs::image::ToDynamic;
//! use jpegxl_rs::decoder_builder;
//! use image::DynamicImage;
//!
//! let sample = std::fs::read("test/sample.jxl")?;
//! let decoder = decoder_builder().build()?;
//! let img = decoder.decode(&sample)?.into_dynamic_image();       
//! # Ok(()) };
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

#[macro_use]
extern crate derive_builder;

mod common;
pub mod decode;
pub mod encode;
mod errors;
pub mod memory;
pub mod parallel;

#[cfg(feature = "image")]
pub mod image;

pub use common::Endianness;
pub use decode::decoder_builder;
pub use encode::encoder_builder;
pub use errors::{DecodeError, EncodeError};

#[cfg(feature = "threads")]
pub use parallel::ThreadsRunner;
