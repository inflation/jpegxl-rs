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
//! # || -> Result<(), Box<dyn std::error::Error>> {
//! use jpegxl_rs::{parallel::*, decoder::*};
//! // Use the default C++ Threadpool runner:
//! let mut parallel_runner = Box::new(ThreadsRunner::default());
//! let mut decoder: JXLDecoder<u8> = decoder_builder().parallel_runner(parallel_runner).build();
//! # Ok(())
//! # };
//! ```

use std::ffi::c_void;

#[cfg(feature = "with-rayon")]
pub mod rayon_runner;
#[cfg(not(feature = "without-threads"))]
pub mod threads_runner;

#[cfg(feature = "with-rayon")]
pub use rayon_runner::*;
#[cfg(not(feature = "without-threads"))]
pub use threads_runner::*;

/// Parallel runner return code
pub use jpegxl_sys::JxlParallelRetCode;
/// Parallel runner initialization callback type
pub type InitFn = unsafe extern "C" fn(*mut c_void, u64) -> i32;
/// Parallel runner data processing callback type
pub type RunFn = unsafe extern "C" fn(*mut c_void, u32, u64);

/// JxlParallelRunner function type
pub type ParallelRunnerFn = unsafe extern "C" fn(
    runner_opaque: *mut c_void,
    jpegxl_opaque: *mut c_void,
    init_func: Option<InitFn>,
    run_func: Option<RunFn>,
    start_range: u32,
    end_range: u32,
) -> JxlParallelRetCode;

/// JPEG XL Parallel Runner
pub trait JXLParallelRunner: std::fmt::Debug {
    /// FFI runner function.
    /// Check `jpeg-xl` header files for more explainations.
    fn runner(&self) -> ParallelRunnerFn;

    /// Helper function to get an opaque pointer
    fn as_opaque_ptr(&mut self) -> *mut c_void {
        self as *mut Self as _
    }
}

#[allow(unreachable_code)]
pub(crate) fn choose_runner() -> Option<Box<dyn JXLParallelRunner>> {
    #[cfg(not(feature = "without-threads"))]
    return Some(Box::new(ThreadsRunner::default()));

    #[cfg(feature = "with-rayon")]
    return Some(Box::new(RayonRunner::default()));

    None
}
