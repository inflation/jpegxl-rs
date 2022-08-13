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
//! use jpegxl_rs::{decoder_builder, parallel::*};
//! // Use the default C++ Threadpool runner:
//! let mut parallel_runner = ThreadsRunner::default();
//! let mut decoder = decoder_builder().parallel_runner(&parallel_runner).build()?;
//! # Ok(())
//! # };
//! ```

#[cfg(feature = "threads")]
pub mod threads_runner;

use std::ffi::c_void;

#[cfg(feature = "threads")]
pub use threads_runner::*;

/// Parallel runner return code
pub use jpegxl_sys::parallel_runner::JxlParallelRetCode;
/// Parallel runner initialization callback type
pub type InitFn = unsafe extern "C" fn(*mut c_void, usize) -> i32;
/// Parallel runner data processing callback type
pub type RunFn = unsafe extern "C" fn(*mut c_void, u32, usize);

/// [`JxlParallelRunner`] function type
pub type RunnerFn = unsafe extern "C" fn(
    runner_opaque: *mut c_void,
    jpegxl_opaque: *mut c_void,
    init_func: InitFn,
    run_func: RunFn,
    start_range: u32,
    end_range: u32,
) -> JxlParallelRetCode;

/// JPEG XL Parallel Runner
pub trait JxlParallelRunner {
    /// FFI runner function.
    /// Check `jpeg-xl` header files for more explanations.
    fn runner(&self) -> RunnerFn;

    /// Get an opaque pointer to the runner.
    fn as_opaque_ptr(&self) -> *mut c_void;
}
