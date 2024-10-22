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

//! Interface to allow the injection of different color management systems
//! (CMSes, also called color management modules, or CMMs) in JPEG XL.
//!
//! A CMS is needed by the JPEG XL encoder and decoder to perform colorspace
//! conversions. This defines an interface that can be implemented for different
//! CMSes and then passed to the library.

use std::ffi::c_void;

use crate::common::types::JxlBool;

use super::color_encoding::JxlColorEncoding;

pub type JpegXlCmsSetFieldsFromIccFunc = extern "C-unwind" fn(
    user_data: *mut c_void,
    icc_data: *const u8,
    icc_size: usize,
    c: *mut JxlColorEncoding,
    cmyk: *mut JxlBool,
) -> JxlBool;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlColorProfileIcc {
    data: *const u8,
    size: usize,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlColorProfile {
    pub icc: JxlColorProfileIcc,
    pub color_encoding: JxlColorEncoding,
    pub num_channels: usize,
}

pub type JpegXlCmsInitFunc = extern "C-unwind" fn(
    init_data: *mut c_void,
    num_threads: usize,
    pixels_per_thread: usize,
    input_profile: *const JxlColorProfile,
    output_profile: *const JxlColorProfile,
    intensity_target: f32,
) -> *mut c_void;

pub type JpegXlCmsGetBufferFunc =
    extern "C-unwind" fn(user_data: *mut c_void, thread: usize) -> *mut f32;

pub type JpegXlCmsRunFunc = extern "C-unwind" fn(
    user_data: *mut c_void,
    thread: usize,
    input_buffer: *const f32,
    output_buffer: *mut f32,
    num_pixels: usize,
) -> JxlBool;

pub type JpegXlCmsDestroyFun = extern "C-unwind" fn(user_data: *mut c_void);

#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlCmsInterface {
    pub set_fields_data: *mut c_void,
    pub set_fields_from_icc: JpegXlCmsSetFieldsFromIccFunc,
    pub init_data: *mut c_void,
    pub init: JpegXlCmsInitFunc,
    pub get_src_buf: JpegXlCmsGetBufferFunc,
    pub get_dst_buf: JpegXlCmsGetBufferFunc,
    pub run: JpegXlCmsRunFunc,
    pub destroy: JpegXlCmsDestroyFun,
}
