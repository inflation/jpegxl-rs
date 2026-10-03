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

//! Wrapper for default thread pool implementation with C++ standard library

use std::{ffi::c_void, ptr::null_mut};

#[allow(clippy::wildcard_imports)]
use jpegxl_sys::threads::thread_parallel_runner::*;

use super::{JxlParallelRunner, ParallelRunner};

use crate::memory::MemoryManager;

/// Wrapper for default thread pool implementation with C++ standard library
pub struct ThreadsRunner<'mm> {
    runner_ptr: *mut c_void,
    _memory_manager: Option<&'mm dyn MemoryManager>,
}

impl<'mm> ThreadsRunner<'mm> {
    /// Construct with number of threads
    #[must_use]
    pub fn new(
        memory_manager: Option<&'mm dyn MemoryManager>,
        num_workers: Option<usize>,
    ) -> Option<Self> {
        let mm = memory_manager.map(MemoryManager::manager);
        // SAFETY: libjxl copies the memory manager, so `mm` only has to outlive the call
        let runner_ptr = unsafe {
            JxlThreadParallelRunnerCreate(
                mm.as_ref().map_or(null_mut(), |mm| mm),
                num_workers.unwrap_or_else(|| JxlThreadParallelRunnerDefaultNumWorkerThreads()),
            )
        };

        if runner_ptr.is_null() {
            None
        } else {
            Some(Self {
                runner_ptr,
                _memory_manager: memory_manager,
            })
        }
    }
}

impl Default for ThreadsRunner<'_> {
    /// # Panics
    /// Panics if libjxl fails to create the runner
    fn default() -> Self {
        Self::new(None, None).expect("failed to create the parallel runner")
    }
}

impl ParallelRunner for ThreadsRunner<'_> {
    fn runner(&self) -> JxlParallelRunner {
        JxlThreadParallelRunner
    }

    fn as_opaque_ptr(&self) -> *mut c_void {
        self.runner_ptr
    }
}

impl Drop for ThreadsRunner<'_> {
    fn drop(&mut self) {
        // SAFETY: `self.runner_ptr` is valid and never used again
        unsafe { JxlThreadParallelRunnerDestroy(self.runner_ptr) };
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::tests::BumpManager;

    use super::*;

    #[test]
    fn test_construction() {
        let memory_manager = BumpManager::new(1024);
        let parallel_runner = ThreadsRunner::new(Some(&memory_manager), Some(10));
        assert!(parallel_runner.is_some());
    }
}
