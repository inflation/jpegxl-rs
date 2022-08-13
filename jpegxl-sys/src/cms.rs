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

use crate::{JxlBool, JxlColorEncoding};

#[repr(C)]
pub struct JxlColorProfileIcc {
    pub data: *const u8,
    pub size: usize,
}

#[repr(C)]
pub struct JxlColorProfile {
    pub icc: JxlColorProfileIcc,
    pub color_encoding: JxlColorEncoding,
    pub num_channels: usize,
}

pub type JpegXlCmsInitFunc = extern "C" fn(
    init_data: *mut c_void,
    num_threads: usize,
    pixels_per_thread: usize,
    input_profile: *const JxlColorProfile,
    output_profile: *const JxlColorProfile,
    intensity_target: f32,
) -> *mut c_void;

pub type JpegXlCmsGetBufferFunc = extern "C" fn(user_data: *mut c_void, thread: usize) -> *mut f32;

pub type JpegXlCmsRunFunc = extern "C" fn(
    user_data: *mut c_void,
    thread: usize,
    input_buffer: *const f32,
    output_buffer: *mut f32,
    num_pixels: usize,
) -> JxlBool;

pub type JpegXlCmsDestroyFun = extern "C" fn(user_data: *mut c_void);

#[repr(C)]
pub struct JxlCmsInterface {
    pub init_data: *mut c_void,
    pub init: JpegXlCmsInitFunc,
    pub get_src_buf: JpegXlCmsGetBufferFunc,
    pub get_dst_buf: JpegXlCmsGetBufferFunc,
    pub run: JpegXlCmsRunFunc,
    pub destroy: JpegXlCmsDestroyFun,
}
