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

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![doc = include_str!("../README.md")]

#[macro_use]
extern crate derive_builder;

mod common;
pub mod decode;
pub mod encode;
mod errors;
pub mod memory;
pub mod parallel;
pub mod utils;

#[cfg(feature = "image")]
pub mod image;

#[cfg(test)]
mod tests;

pub use common::Endianness;
pub use decode::decoder_builder;
pub use encode::encoder_builder;
pub use errors::{DecodeError, EncodeError};

pub use parallel::resizable_runner::ResizableRunner;
pub use parallel::threads_runner::ThreadsRunner;
