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

#![deny(missing_docs)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![deny(clippy::all)]

//! # Overview
//! A safe JPEGXL wrapper over `jpeg-xl` library. Check out the original [library](https://gitlab.com/wg1/jpeg-xl)
//! and the [bindings](https://github.com/inflation/jpegxl-sys).

//! # Usage

//! ## Decoding
//! ```rust
//! # || -> Result<(), Box<dyn std::error::Error>> {  
//! use jpegxl_rs::*;
//!
//! let sample = std::fs::read("test/sample.jxl")?;
//! let mut decoder: JXLDecoder<u8> = decoder_builder().build();
//! let (info, buffer) = decoder.decode(&sample)?;
//! # Ok(()) };
//! ```

//! Set output pixel paramaters
//! ```rust
//! // Pixel type is set by type parameter
//! # || -> Result<(), Box<dyn std::error::Error>> {
//! # use jpegxl_rs::*;
//! let mut decoder: JXLDecoder<u16> = decoder_builder()
//!                                     .num_channels(3)
//!                                     .endian(Endianness::Big)
//!                                     .align(8)
//!                                     .build();
//! # Ok(()) };
//! ```

//! ## Encoding
//! ```rust
//! # use jpegxl_rs::encoder::*;
//! # || -> Result<(), Box<dyn std::error::Error>> {
//! use image::io::Reader as ImageReader;
//!
//! let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgba16();
//! let mut encoder = encoder_builder().build();
//! let buffer: Vec<u8> = encoder.encode(
//!                         &sample,
//!                         sample.width() as u64,
//!                         sample.height() as u64
//!                       )?;
//! # Ok(()) };
//! ```

//! ## [`image`](https://crates.io/crates/image) crate integration
//! The integration is enabled by default. If you don't need it, disable `with-image` feature.
//! ```
//! # || -> Result<(), Box<dyn std::error::Error>> {
//! use jpegxl_rs::image::*;
//!
//! let sample = std::fs::read("test/sample.jxl")?;
//! let decoder: JXLImageDecoder<u16> = JXLImageDecoder::new(&sample)?;
//! let img = image::DynamicImage::from_decoder(decoder)?;       
//! # Ok(()) };
//! ```

mod common;
pub mod decoder;
pub mod encoder;
pub mod error;
pub mod memory;
pub mod parallel;
pub mod threadpool;

#[cfg(feature = "with-image")]
pub mod image;

pub use common::*;
pub use decoder::*;
pub use encoder::*;
