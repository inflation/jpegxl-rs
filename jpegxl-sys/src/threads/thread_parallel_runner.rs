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

//! Implementation using `std::thread` of a [`JxlParallelRunner`].

//! Implementation of [`JxlParallelRunner`] that can be used to enable
//! multithreading when using the JPEG XL library. This uses `std::thread`
//! internally and related synchronization functions. The number of threads
//! created is fixed at construction time and the threads are re-used for every
//! `ThreadParallelRunner::Runner` call. Only one concurrent
//! `JxlThreadParallelRunner` call per instance is allowed at a time.
//!
//! This is a scalable, lower-overhead thread pool runner, especially suitable
//! for data-parallel computations in the fork-join model, where clients need to
//! know when all tasks have completed.
//!
//! This thread pool can efficiently load-balance millions of tasks using an
//! atomic counter, thus avoiding per-task virtual or system calls. With 48
//! hyperthreads and 1M tasks that add to an atomic counter, overall runtime is
//! 10-20x higher when using `std::async`, and ~200x for a queue-based thread
//! pool.

use std::ffi::c_void;

#[cfg(doc)]
use super::parallel_runner::JxlParallelRunner;
use super::parallel_runner::{JxlParallelRetCode, JxlParallelRunFunction, JxlParallelRunInit};
use crate::common::memory_manager::JxlMemoryManager;

extern "C-unwind" {
    /// Parallel runner internally using `std::thread`. Use as [`JxlParallelRunner`].
    pub fn JxlThreadParallelRunner(
        runner_opaque: *mut c_void,
        jpegxl_opaque: *mut c_void,
        init: JxlParallelRunInit,
        func: JxlParallelRunFunction,
        start_range: u32,
        end_range: u32,
    ) -> JxlParallelRetCode;

    /// Creates the runner for [`JxlThreadParallelRunner`]. Use as the opaque runner.
    pub fn JxlThreadParallelRunnerCreate(
        memory_manager: *const JxlMemoryManager,
        num_worker_threads: usize,
    ) -> *mut c_void;

    /// Destroys the runner created by [`JxlThreadParallelRunnerCreate`].
    pub fn JxlThreadParallelRunnerDestroy(runner_opaque: *mut c_void);

    /// Returns a default `num_worker_threads` value for [`JxlThreadParallelRunnerCreate`].
    pub fn JxlThreadParallelRunnerDefaultNumWorkerThreads() -> usize;
}
