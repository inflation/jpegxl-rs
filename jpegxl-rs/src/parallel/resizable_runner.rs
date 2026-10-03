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

//! Wrapper for resizable thread pool implementation with C++ standard library

use std::{ffi::c_void, ptr::null_mut};

use jpegxl_sys::threads::resizable_parallel_runner as api;

use super::{JxlParallelRunner, ParallelRunner};

use crate::{decode::BasicInfo, memory::MemoryManager};

/// Wrapper for resizable thread pool implementation with C++ standard library
pub struct ResizableRunner<'mm> {
    runner_ptr: *mut c_void,
    _memory_manager: Option<&'mm dyn MemoryManager>,
}

impl<'mm> ResizableRunner<'mm> {
    /// Construct with number of threads
    #[must_use]
    pub fn new(memory_manager: Option<&'mm dyn MemoryManager>) -> Option<Self> {
        let mm = memory_manager.map(MemoryManager::manager);
        // SAFETY: libjxl copies the memory manager, so `mm` only has to outlive the call
        let runner_ptr = unsafe {
            api::JxlResizableParallelRunnerCreate(mm.as_ref().map_or(null_mut(), |mm| mm))
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

    /// Set number of threads depending on the size of the image
    pub fn set_num_threads(&self, width: u64, height: u64) {
        // SAFETY: this is a pure function of its arguments
        let num = unsafe { api::JxlResizableParallelRunnerSuggestThreads(width, height) };
        // SAFETY: `self.runner_ptr` is valid until drop
        unsafe { api::JxlResizableParallelRunnerSetThreads(self.runner_ptr, num as usize) };
    }
}

impl Default for ResizableRunner<'_> {
    /// # Panics
    /// Panics if libjxl fails to create the runner
    fn default() -> Self {
        Self::new(None).expect("failed to create the parallel runner")
    }
}

impl ParallelRunner for ResizableRunner<'_> {
    fn runner(&self) -> JxlParallelRunner {
        api::JxlResizableParallelRunner
    }

    fn as_opaque_ptr(&self) -> *mut c_void {
        self.runner_ptr
    }

    fn callback_basic_info(&self, info: &BasicInfo) {
        self.set_num_threads(info.xsize.into(), info.ysize.into());
    }
}

impl Drop for ResizableRunner<'_> {
    fn drop(&mut self) {
        // SAFETY: `self.runner_ptr` is valid and never used again
        unsafe { api::JxlResizableParallelRunnerDestroy(self.runner_ptr) };
    }
}

#[cfg(test)]
mod tests {

    use crate::memory::tests::BumpManager;

    use super::*;

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn test_construction() {
        let memory_manager = BumpManager::new(1024);
        let parallel_runner = ResizableRunner::new(Some(&memory_manager));
        assert!(parallel_runner.is_some());
    }
}
