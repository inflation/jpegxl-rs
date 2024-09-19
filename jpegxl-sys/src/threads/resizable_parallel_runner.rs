/*!
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

//! Implementation using `std::thread` of a resizeable [`JxlParallelRunner`].

//! Implementation of [`JxlParallelRunner`] than can be used to enable
//! multithreading when using the JPEG XL library. This uses `std::thread`
//! internally and related synchronization functions. The number of threads
//! created can be changed after creation of the thread pool; the threads
//! (including the main thread) are re-used for every
//! `ResizableParallelRunner::Runner` call. Only one concurrent
//! [`JxlResizableParallelRunner`] call per instance is allowed at a time.
//!
//! This is a scalable, lower-overhead thread pool runner, especially suitable
//! for data-parallel computations in the fork-join model, where clients need to
//! know when all tasks have completed.
//!
//! Compared to the implementation in [`super::thread_parallel_runner`], this
//! implementation is tuned for execution on lower-powered systems, including
//! for example ARM CPUs with big.LITTLE computation models.

use std::ffi::c_void;

#[cfg(doc)]
use super::parallel_runner::JxlParallelRunner;
use super::parallel_runner::{JxlParallelRetCode, JxlParallelRunFunction, JxlParallelRunInit};
use crate::common::memory_manager::JxlMemoryManager;

extern "C-unwind" {
    /// Parallel runner internally using `std::thread`. Use as [`JxlParallelRunner`].
    pub fn JxlResizableParallelRunner(
        runner_opaque: *mut c_void,
        jpegxl_opaque: *mut c_void,
        init: JxlParallelRunInit,
        func: JxlParallelRunFunction,
        start_range: u32,
        end_range: u32,
    ) -> JxlParallelRetCode;

    /// Creates the runner for [`JxlResizableParallelRunner`]. Use as the opaque
    /// runner. The runner will execute tasks on the calling thread until
    /// [`JxlResizableParallelRunnerSetThreads`] is called.
    pub fn JxlResizableParallelRunnerCreate(memory_manager: *const JxlMemoryManager)
        -> *mut c_void;

    /// Changes the number of threads for [`JxlResizableParallelRunner`].
    pub fn JxlResizableParallelRunnerSetThreads(runner_opaque: *mut c_void, num_threads: usize);

    /// Suggests a number of threads to use for an image of given size.
    pub fn JxlResizableParallelRunnerSuggestThreads(xsize: u64, ysize: u64) -> u32;

    /// Destroys the runner created by [`JxlResizableParallelRunnerCreate`].
    pub fn JxlResizableParallelRunnerDestroy(runner_opaque: *mut c_void);
}
