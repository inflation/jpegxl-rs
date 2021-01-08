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

//! Example implementation of a multithread runner
//! # Example
//! ```
//! # use jpegxl_rs::{parallel::*, decoder::*};
//! # || -> Result<(), Box<dyn std::error::Error>> {
//! let mut parallel_runner = Box::new(ParallelRunner::default());
//! let mut decoder: JXLDecoder<u8> = decoder_builder().parallel_runner(parallel_runner).build();
//! # Ok(())
//! # };
//! ```

use jpegxl_sys::*;
use num_cpus;
use std::ffi::c_void;
use std::thread;

pub use jpegxl_sys::JxlParallelRetCode;

use crate::memory::JXLMemoryManager;

/// Parallel run initialization callback type
pub type InitFn = unsafe extern "C" fn(*mut c_void, u64) -> i32;
/// Parallel run data processing callback type
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
        self as *mut Self as *mut c_void
    }
}

/// Example implementation of a multithread runner
#[derive(Debug)]
pub struct ParallelRunner {
    num_threads: usize,
}

impl ParallelRunner {
    /// Create a multithread parallel runner with specific thread number
    pub fn new(num_threads: usize) -> Self {
        ParallelRunner { num_threads }
    }
}

impl JXLParallelRunner for ParallelRunner {
    /// Divide the task into chunks, then spawn a thread for each chunk.
    /// Since the library explicitly states that there is no communications between each call,
    /// we can safely ignore synchronizations.

    fn runner(&self) -> ParallelRunnerFn {
        // #[no_mangle]
        unsafe extern "C" fn runner_func(
            runner_opaque: *mut c_void,
            jpegxl_opaque: *mut c_void,
            init_func: Option<InitFn>,
            run_func: Option<RunFn>,
            start_range: u32,
            end_range: u32,
        ) -> JxlParallelRetCode {
            let runner = (runner_opaque as *mut ParallelRunner).as_ref().unwrap();
            let ret_code = init_func.unwrap()(jpegxl_opaque, runner.num_threads as size_t);
            if ret_code != 0 {
                return ret_code;
            };

            let chunk_size =
                ((end_range - start_range) as f64 / runner.num_threads as f64).ceil() as usize;
            (start_range..end_range)
                .collect::<Vec<u32>>()
                .chunks(chunk_size)
                .enumerate()
                .map(|(id, chunk)| {
                    let ch = Vec::from(chunk);
                    let ptr = jpegxl_opaque as usize; // Bypass Send check
                    thread::spawn(move || {
                        for i in ch {
                            // Pass current job index and thread id
                            run_func.unwrap()(ptr as *mut c_void, i, id as size_t);
                        }
                    })
                })
                .for_each(|t| t.join().unwrap());

            return 0;
        }
        runner_func
    }
}

impl Default for ParallelRunner {
    /// Create a multithread parallel runner with one thread per core
    fn default() -> Self {
        Self::new(num_cpus::get())
    }
}

#[cfg(not(feature = "without-threads"))]
#[derive(Debug)]
/// Wrapper for default threadspool implementation with C++ standard library
pub struct ThreadsRunner {
    runner_ptr: *mut c_void,
}

#[cfg(not(feature = "without-threads"))]
impl ThreadsRunner {
    /// Construct with number of threads
    pub fn new(memory_manager: Option<&mut dyn JXLMemoryManager>, num_workers: usize) -> Self {
        let memory_manager = memory_manager
            .map(|s| &mut s.to_manager() as *const JxlMemoryManager)
            .unwrap_or(std::ptr::null());
        Self {
            runner_ptr: unsafe {
                JxlThreadParallelRunnerCreate(memory_manager, num_workers as u64)
            },
        }
    }

    /// Defualt
    pub fn default() -> Self {
        Self {
            runner_ptr: unsafe {
                JxlThreadParallelRunnerCreate(
                    std::ptr::null(),
                    JxlThreadParallelRunnerDefaultNumWorkerThreads(),
                )
            },
        }
    }
}

#[cfg(not(feature = "without-threads"))]
impl JXLParallelRunner for ThreadsRunner {
    fn runner(&self) -> ParallelRunnerFn {
        JxlThreadParallelRunner
    }

    fn as_opaque_ptr(&mut self) -> *mut c_void {
        self.runner_ptr
    }
}

#[cfg(not(feature = "without-threads"))]
impl Drop for ThreadsRunner {
    fn drop(&mut self) {
        unsafe { JxlThreadParallelRunnerDestroy(self.runner_ptr) };
    }
}
