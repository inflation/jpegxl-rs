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

use crate::common::{JxlMemoryManager, JxlParallelRunner, JxlPixelFormat};

// Opaque type
#[repr(C)]
pub struct JxlButteraugliApi {
    _unused: [u8; 0],
}

// Opaque type
#[repr(C)]
pub struct JxlButteraugliResult {
    _unused: [u8; 0],
}

extern "C" {
    pub fn JxlButteraugliResultDestroy(result: *mut JxlButteraugliResult);

    pub fn JxlButteraugliApiCreate(
        memory_manager: *const JxlMemoryManager,
    ) -> *mut JxlButteraugliApi;

    pub fn JxlButteraugliApiSetParallelRunner(
        api: *mut JxlButteraugliApi,
        parallel_runner: JxlParallelRunner,
        parallel_runner_opaque: *mut c_void,
    );

    pub fn JxlButteraugliApiSetHFAsymmetry(api: *mut JxlButteraugliApi, v: f32);

    pub fn JxlButteraugliApiSetIntensityTarget(api: *mut JxlButteraugliApi, v: f32);

    pub fn JxlButteraugliApiDestroy(api: *mut JxlButteraugliApi);

    pub fn JxlButteraugliCompute(
        api: *const JxlButteraugliApi,
        xsize: u32,
        ysize: u32,
        pixel_format_orig: *const JxlPixelFormat,
        buffer_orig: *const c_void,
        size_orig: usize,
        pixel_format_dist: *const JxlPixelFormat,
        buffer_dist: *const c_void,
        size_dist: usize,
    ) -> *mut JxlButteraugliResult;

    pub fn JxlButteraugliResultGetMaxDistance(result: *const JxlButteraugliResult) -> f32;

    pub fn JxlButteraugliResultGetDistance(result: *const JxlButteraugliResult, pnorm: f32) -> f32;

    pub fn JxlButteraugliResultGetDistmap(
        result: *const JxlButteraugliResult,
        buffer: *const *const f32,
        row_stride: *mut u32,
    );
}
