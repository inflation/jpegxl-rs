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

use rayon::*;

use crate::parallel::*;

// Sending pointer across threads
#[derive(Debug, Clone, Copy)]
struct CPointer(*mut std::ffi::c_void);

unsafe impl Send for CPointer {}

/// Example implementation of a multithread runner
#[cfg(not(feature = "without-rayon"))]
#[derive(Debug)]
pub struct RayonRunner {
    pool: ThreadPool,
    runner_func: ParallelRunnerFn,
}

#[cfg(not(feature = "without-rayon"))]
impl RayonRunner {
    /// Create a multithread parallel runner with specific thread number
    pub fn new(num_threads: Option<usize>) -> Self {
        let mut pool_builder = ThreadPoolBuilder::new();
        if let Some(num) = num_threads {
            pool_builder = pool_builder.num_threads(num);
        }
        let pool = pool_builder.build().unwrap();

        // Divide the task into chunks, then feed them to the threadpool
        unsafe extern "C" fn runner_func(
            runner_opaque: *mut c_void,
            jpegxl_opaque: *mut c_void,
            init_func: Option<InitFn>,
            run_func: Option<RunFn>,
            start_range: u32,
            end_range: u32,
        ) -> JxlParallelRetCode {
            let runner = (runner_opaque as *mut RayonRunner).as_ref().unwrap();
            let pool = &runner.pool;
            let num_threads = pool.current_num_threads();
            let ret_code = init_func.unwrap()(jpegxl_opaque, num_threads as u64);
            if ret_code != 0 {
                return ret_code;
            };

            let mut chunk_size =
                ((end_range - start_range) as f64 / num_threads as f64).ceil() as usize;

            let ptr = CPointer(jpegxl_opaque);
            (start_range..end_range)
                .collect::<Vec<u32>>()
                .chunks(chunk_size)
                .for_each(|s| {
                    pool.install(move || {
                        let id = pool.current_thread_index().unwrap() as u64;
                        for i in s {
                            run_func.unwrap()(ptr.0, *i, id);
                        }
                    });
                });

            0
        }

        RayonRunner { pool, runner_func }
    }
}

#[cfg(not(feature = "without-rayon"))]
impl JXLParallelRunner for RayonRunner {
    fn runner(&self) -> ParallelRunnerFn {
        self.runner_func
    }
}

#[cfg(not(feature = "without-rayon"))]
impl Default for RayonRunner {
    /// Create a multithread parallel runner with one thread per core
    fn default() -> Self {
        Self::new(None)
    }
}
