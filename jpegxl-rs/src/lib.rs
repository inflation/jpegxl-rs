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
//! and the [bindings](https://github.com/inflation/jpegxl-rs/tree/master/jpegxl-sys).
//!
//! # Building
//!
//! If you wish to specify a custom library path, set the `DEP_JXL_LIB` environment variable.
//!
//! Building `libjxl` and statically linking can be enabled by using the `vendored` feature.
//!
//! If you don't want to depend on C++ standard library, disable the feature `threads`.
//!
//! # Usage
//!
//! Currently, `u8`, `u16`, `f16` and `f32` are supported as pixel types.
//!
//! ## Decoding
//!
//! ```
//! # use jpegxl_rs::*;
//! # use jpegxl_rs::decode::{DecoderResult, Data};
//! # || -> Result<(), Box<dyn std::error::Error>> {
//! # let sample = [];
//! let mut decoder = decoder_builder().build()?;
//! let DecoderResult { width, height, data ,..} = decoder.decode(&sample)?;
//! match data {
//!     Data::U8(data) => { /* do something with Vec<u8> data */ },
//!     Data::U16(data) => { /* do something with Vec<u16> data */ },
//!     Data::F16(data) => { /* do something with Vec<f16> data */ },
//!     Data::F32(data) => { /* do something with Vec<f32> data */ },
//! }
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
//! let sample = ImageReader::open("../samples/sample.png")?.decode()?.to_rgba16();
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
//! The integration is enabled by default. If you don't need it, disable `image` feature.
//!
//! ```
//! use jpegxl_rs::image::ToDynamic;
//! use jpegxl_rs::decoder_builder;
//! use image::DynamicImage;
//!
//! let sample = std::fs::read("../samples/sample.jxl")?;
//! let decoder = decoder_builder().build()?;
//! let img = decoder.decode(&sample)?.into_dynamic_image();       
//! # Ok::<(), Box<dyn std::error::Error>>(())
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

#[cfg(test)]
mod tests;

pub use common::Endianness;
pub use decode::decoder_builder;
pub use encode::encoder_builder;
pub use errors::{DecodeError, EncodeError};

#[cfg(feature = "threads")]
pub use parallel::resizable_runner::ResizableRunner;
#[cfg(feature = "threads")]
pub use parallel::threads_runner::ThreadsRunner;
