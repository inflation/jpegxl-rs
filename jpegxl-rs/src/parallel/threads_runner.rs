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

#![cfg_attr(docsrs, doc(cfg(feature = "threads")))]

use std::{ffi::c_void, ptr::null_mut};

#[allow(clippy::wildcard_imports)]
use jpegxl_sys::parallel_runner::*;

use super::{JxlParallelRunner, RunnerFn};

use crate::memory::JxlMemoryManager;

/// Wrapper for default threadpool implementation with C++ standard library
pub struct ThreadsRunner<'mm> {
    runner_ptr: *mut c_void,
    _memory_manager: Option<&'mm dyn JxlMemoryManager>,
}

impl<'mm> ThreadsRunner<'mm> {
    /// Construct with number of threads
    #[must_use]
    pub fn new(
        memory_manager: Option<&'mm dyn JxlMemoryManager>,
        num_workers: Option<usize>,
    ) -> Option<Self> {
        let mm = memory_manager.map(JxlMemoryManager::manager);
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
    fn default() -> Self {
        Self {
            runner_ptr: unsafe {
                JxlThreadParallelRunnerCreate(
                    std::ptr::null(),
                    JxlThreadParallelRunnerDefaultNumWorkerThreads(),
                )
            },
            _memory_manager: None,
        }
    }
}

impl JxlParallelRunner for ThreadsRunner<'_> {
    fn runner(&self) -> RunnerFn {
        JxlThreadParallelRunner
    }

    fn as_opaque_ptr(&self) -> *mut c_void {
        self.runner_ptr
    }
}

impl Drop for ThreadsRunner<'_> {
    fn drop(&mut self) {
        unsafe { JxlThreadParallelRunnerDestroy(self.runner_ptr) };
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::tests::{BumpManager, NoManager};

    use super::*;

    #[test]
    fn test_construction() {
        let memory_manager = NoManager {};
        let parallel_runner = ThreadsRunner::new(Some(&memory_manager), None);
        assert!(parallel_runner.is_none());

        let memory_manager = BumpManager::<1024>::default();
        let parallel_runner = ThreadsRunner::new(Some(&memory_manager), None);
        assert!(parallel_runner.is_some());
    }
}
