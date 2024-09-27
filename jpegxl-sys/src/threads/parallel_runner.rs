/*
This file is part of jpegxl-sys.

jpegxl-sys is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

jpegxl-sys is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with jpegxl-sys.  If not, see <https://www.gnu.org/licenses/>.
*/

//! API for running data operations in parallel in a multi-threaded environment.
//! This module allows the JPEG XL caller to define their own way of creating and
//! assigning threads.
//!
//! The [`JxlParallelRunner`] function type defines a parallel data processing
//! runner that may be implemented by the caller to allow the library to process
//! in multiple threads. The multi-threaded processing in this library only
//! requires to run the same function over each number of a range, possibly
//! running each call in a different thread. The JPEG XL caller is responsible
//! for implementing this logic using the thread APIs available in their system.
//! For convenience, a C++ implementation based on `std::thread` is provided in
//! [`super::thread_parallel_runner`] (part of the `jpegxl_threads` library).
//!
//! Thread pools usually store small numbers of heterogeneous tasks in a queue.
//! When tasks are identical or differ only by an integer input parameter, it is
//! much faster to store just one function of an integer parameter and call it
//! for each value. Conventional vector-of-tasks can be run in parallel using a
//! lambda function adapter that simply calls `task_funcs[task]`.
//!
//! If no multi-threading is desired, a `NULL` value of [`JxlParallelRunner`]
//! will use an internal implementation without multi-threading.

use std::ffi::{c_int, c_void};

/// Return code used in the `JxlParallel*` functions as return value. A value
/// of [`JXL_PARALLEL_RET_SUCCESS`] means success and any other value means error.
/// The special value [`JXL_PARALLEL_RET_RUNNER_ERROR`] can be used by the runner
/// to indicate any other error.
pub type JxlParallelRetCode = c_int;

/// Code returned by the [`JxlParallelRunInit`] function to indicate success.
pub const JXL_PARALLEL_RET_SUCCESS: JxlParallelRetCode = 0;

/// Code returned by the [`JxlParallelRunInit`] function to indicate a general error.
pub const JXL_PARALLEL_RET_RUNNER_ERROR: JxlParallelRetCode = -1;

/// Parallel run initialization callback. See [`JxlParallelRunner`] for details.
///
/// This function MUST be called by the [`JxlParallelRunner`] only once, on the
/// same thread that called [`JxlParallelRunner`], before any parallel
/// execution. The purpose of this call is to provide the maximum number of
/// threads that the [`JxlParallelRunner`] will use, which can be used by JPEG XL
/// to allocate per-thread storage if needed.
///
/// # Parameters
/// - `jpegxl_opaque`: the `jpegxl_opaque` handle provided to
///   [`JxlParallelRunner`] must be passed here.
/// - `num_threads`: the maximum number of threads. This value must be
///   positive.
///
/// # Returns
/// - `0` if the initialization process was successful.
/// - An error code if there was an error, which should be returned by
///   [`JxlParallelRunner`].
pub type JxlParallelRunInit = unsafe extern "C-unwind" fn(
    jpegxl_opaque: *mut c_void,
    num_threads: usize,
) -> JxlParallelRetCode;

/// Parallel run data processing callback. See [`JxlParallelRunner`] for
/// details.
///
/// This function MUST be called once for every number in the range `[start_range,
/// end_range)` (including `start_range` but not including `end_range`) passing this
/// number as the `value`. Calls for different values may be executed from
/// different threads in parallel.
///
/// # Parameters
/// - `jpegxl_opaque`: the `jpegxl_opaque` handle provided to
///   [`JxlParallelRunner`] must be passed here.
/// - `value`: the number in the range `[start_range, end_range)` of the call.
/// - `thread_id`: the thread number where this function is being called from.
///   This must be lower than the `num_threads` value passed to
///   [`JxlParallelRunInit`].
pub type JxlParallelRunFunction =
    unsafe extern "C-unwind" fn(jpegxl_opaque: *mut c_void, value: u32, thread_id: usize);

/// [`JxlParallelRunner`] function type. A parallel runner implementation can be
/// provided by a JPEG XL caller to allow running computations in multiple
/// threads. This function must call the initialization function [`init`](JxlParallelRunInit) in the
/// same thread that called it and then call the passed [`func`](JxlParallelRunFunction) once for every
/// number in the range `[start_range, end_range)` (including `start_range` but not
/// including `end_range`) possibly from different multiple threads in parallel.
///
/// The [`JxlParallelRunner`] function does not need to be re-entrant. This
/// means that the same [`JxlParallelRunner`] function with the same
/// `runner_opaque` provided parameter will not be called from the library from
/// either [`init`](JxlParallelRunInit) or [`func`](JxlParallelRunFunction) in the same decoder or encoder instance.
/// However, a single decoding or encoding instance may call the provided [`JxlParallelRunner`] multiple
/// times for different parts of the decoding or encoding process.
///
/// # Returns
/// - `0`: if the [`init`](JxlParallelRunInit) call succeeded (returned `0`) and no other error
///   occurred in the runner code.
/// - [`JXL_PARALLEL_RET_RUNNER_ERROR`]: if an error occurred in the runner
///   code, for example, setting up the threads.
/// - The return value of [`init()`](JxlParallelRunInit) if non-zero.
pub type JxlParallelRunner = unsafe extern "C-unwind" fn(
    runner_opaque: *mut c_void,
    jpegxl_opaque: *mut c_void,
    init: JxlParallelRunInit,
    func: JxlParallelRunFunction,
    start_range: u32,
    end_range: u32,
) -> JxlParallelRetCode;
