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
//! # use jpegxl_rs::*;
//! # || -> Result<(), Box<dyn std::error::Error>> {
//! let mut parallel_runner = ParallelRunner::new();
//! let mut decoder = JxlDecoder::new(None, Some(&mut parallel_runner))
//!             .ok_or(JxlError::CannotCreateDecoder)?;
//! # Ok(())
//! # }();
//! ```

use jpegxl_sys::*;
use num_cpus;
use std::ffi::c_void;
use std::thread;

pub use jpegxl_sys::JxlParallelRetCode;

type ParallelRunnerFn = unsafe extern "C" fn(
    runner_opaque: *mut c_void,
    jpegxl_opaque: *mut c_void,
    init_func: Option<unsafe extern "C" fn(*mut c_void, u64) -> i32>,
    run_func: Option<unsafe extern "C" fn(*mut c_void, u32, u64)>,
    start_range: u32,
    end_range: u32,
) -> JxlParallelRetCode;

/// JPEG XL Parallel Runner
pub trait JpegXLParallelRunner {
    /// FFI runner function.
    /// Check `jpeg-xl` header files for more explainations.
    unsafe extern "C" fn runner_func(
        runner_opaque: *mut c_void,
        jpegxl_opaque: *mut c_void,
        init_func: Option<unsafe extern "C" fn(*mut c_void, u64) -> i32>,
        run_func: Option<unsafe extern "C" fn(*mut c_void, u32, u64)>,
        start_range: u32,
        end_range: u32,
    ) -> JxlParallelRetCode;

    /// Helper function to get an opaque pointer
    fn as_opaque_ptr(&mut self) -> *mut c_void {
        self as *mut Self as *mut c_void
    }

    /// Helper function to get runner function
    fn runner(&self) -> ParallelRunnerFn
    where
        Self: Sized,
    {
        Self::runner_func
    }
}

/// Example implementation of a multithread runner
pub struct ParallelRunner {
    num_threads: usize,
}

impl ParallelRunner {
    /// Create a multithread parallel runner with one thread per core
    pub fn new() -> Self {
        Self::new_with_threads(num_cpus::get())
    }

    /// Create a multithread parallel runner with specific thread number
    pub fn new_with_threads(num_threads: usize) -> Self {
        ParallelRunner { num_threads }
    }
}

impl JpegXLParallelRunner for ParallelRunner {
    /// Divide the task into chunks, then spawn a thread for each chunk.
    /// Since the library explicitly states that there is no communications between each call,
    /// we can safely ignore synchronizations.
    #[no_mangle]
    unsafe extern "C" fn runner_func(
        runner_opaque: *mut c_void,
        jpegxl_opaque: *mut c_void,
        init_func: Option<unsafe extern "C" fn(*mut c_void, u64) -> i32>,
        run_func: Option<unsafe extern "C" fn(*mut c_void, u32, u64)>,
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
}
