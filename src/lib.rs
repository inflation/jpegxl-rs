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
//! A safe JPEGXL Decoder wrapper.

mod common;
/// Decoder
pub mod decoder;
/// Encoder
pub mod encoder;
/// Error types and helper functions
pub mod error;
/// Memory manager interface
pub mod memory;
/// Parallel runner interface
pub mod parallel;

#[cfg(feature = "with-image")]
mod image_support;

pub use decoder::*;
pub use encoder::*;

#[cfg(feature = "with-image")]
pub use image_support::*;
