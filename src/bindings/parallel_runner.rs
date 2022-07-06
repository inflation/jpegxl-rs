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

use std::ffi::c_void;

use crate::common::{
    JxlMemoryManager, JxlParallelRetCode, JxlParallelRunFunction, JxlParallelRunInit,
};

extern "C" {
    pub fn JxlThreadParallelRunner(
        runner_opaque: *mut c_void,
        jpegxl_opaque: *mut c_void,
        init: JxlParallelRunInit,
        func: JxlParallelRunFunction,
        start_range: u32,
        end_range: u32,
    ) -> JxlParallelRetCode;

    pub fn JxlThreadParallelRunnerCreate(
        memory_manager: *const JxlMemoryManager,
        num_worker_threads: usize,
    ) -> *mut c_void;

    pub fn JxlThreadParallelRunnerDestroy(runner_opaque: *mut c_void);

    pub fn JxlThreadParallelRunnerDefaultNumWorkerThreads() -> usize;
}
