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

//! Wrapper for default threadpool implementation with C++ standard library

use jpegxl_sys::*;

use crate::memory::JXLMemoryManager;
use crate::parallel::*;

#[derive(Debug)]
/// Wrapper for default threadpool implementation with C++ standard library
pub struct ThreadsRunner {
    runner_ptr: *mut c_void,
}

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
}

#[cfg(not(feature = "without-threads"))]
impl Default for ThreadsRunner {
    fn default() -> Self {
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

impl JXLParallelRunner for ThreadsRunner {
    fn runner(&self) -> ParallelRunnerFn {
        JxlThreadParallelRunner
    }

    fn as_opaque_ptr(&mut self) -> *mut c_void {
        self.runner_ptr
    }
}

impl Drop for ThreadsRunner {
    fn drop(&mut self) {
        unsafe { JxlThreadParallelRunnerDestroy(self.runner_ptr) };
    }
}
