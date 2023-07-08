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

/// Parallel runner return code
pub use jpegxl_sys::parallel_runner::JxlParallelRetCode;

use crate::decode::BasicInfo;
/// Parallel runner initialization callback type
pub type InitFn = unsafe extern "C" fn(*mut c_void, usize) -> i32;
/// Parallel runner data processing callback type
pub type RunFn = unsafe extern "C" fn(*mut c_void, u32, usize);

/// FFI runner function.
///
/// A parallel runner implementation can be
/// provided by a JPEG XL caller to allow running computations in multiple
/// threads. This function must call the initialization function `init_func` in the
/// same thread that called it and then call the passed `run_func` once for every
/// number in the range [`start_range`, `end_range`) (including `start_range`ge but not
/// including `end_range`) possibly from different multiple threads in parallel.
///
/// The `JxlParallelRunner` function does not need to be re-entrant. This means
/// that the same `JxlParallelRunner` function with the same `runner_opaque`
/// provided parameter will not be called from the library from either `init_func` or
/// `run_func` in the same decoder or encoder instance. However, a single decoding
/// or encoding instance may call the provided `JxlParallelRunner` multiple
/// times for different parts of the decoding or encoding process.
///
/// # Returns
/// - `0`: if the @p init call succeeded (returned 0) and no other error
/// occurred in the runner code.
/// - `JXL_PARALLEL_RET_RUNNER_ERROR` if an error occurred in the runner
/// code, for example, setting up the threads.
/// - Return the return value of `init_func` if non-zero.
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
    /// Get a [`RunnerFn`] for the parallel runner.
    fn runner(&self) -> RunnerFn;

    /// Get an opaque pointer to the runner.
    fn as_opaque_ptr(&self) -> *mut c_void;

    /// Callback function after getting basic info
    #[allow(unused_variables)]
    fn callback_basic_info(&self, basic_info: &BasicInfo) {}
}
