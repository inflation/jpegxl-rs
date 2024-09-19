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

//! Parallel runner interface
//! # Example
//! ```
//! #[cfg(feature = "image")]
//! # || -> Result<(), Box<dyn std::error::Error>> {
//! use jpegxl_rs::{decoder_builder, parallel::threads_runner::ThreadsRunner};
//! // Use the default C++ Threads pool runner:
//! let mut parallel_runner = ThreadsRunner::default();
//! let mut decoder = decoder_builder().parallel_runner(&parallel_runner).build()?;
//! # Ok(())
//! # };
//! ```
//!

use std::ffi::c_void;

pub mod resizable_runner;
pub mod threads_runner;

use jpegxl_sys::threads::parallel_runner::JxlParallelRunner;

pub use jpegxl_sys::threads::parallel_runner::{
    JxlParallelRetCode, JxlParallelRunFunction, JxlParallelRunInit,
};

use crate::decode::BasicInfo;

/// JPEG XL Parallel Runner
#[allow(clippy::module_name_repetitions)]
pub trait ParallelRunner {
    /// Get a [`JxlParallelRunner`] for the parallel runner.
    fn runner(&self) -> JxlParallelRunner;

    /// Get an opaque pointer to the runner.
    fn as_opaque_ptr(&self) -> *mut c_void;

    /// Callback function after getting basic info
    #[allow(unused_variables)]
    fn callback_basic_info(&self, basic_info: &BasicInfo) {}
}
