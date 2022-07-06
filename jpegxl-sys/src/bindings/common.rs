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

use std::{ffi::c_void, os::raw::c_int};

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JxlBool {
    True = 1,
    False = 0,
}

impl From<JxlBool> for bool {
    fn from(b: JxlBool) -> Self {
        match b {
            JxlBool::True => true,
            JxlBool::False => false,
        }
    }
}

impl From<bool> for JxlBool {
    fn from(b: bool) -> Self {
        if b {
            JxlBool::True
        } else {
            JxlBool::False
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JxlDataType {
    Float = 0,
    Boolean,
    Uint8,
    Uint16,
    Uint32,
    Float16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JxlEndianness {
    Native = 0,
    Little,
    Big,
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JxlPixelFormat {
    pub num_channels: u32,
    pub data_type: JxlDataType,
    pub endianness: JxlEndianness,
    pub align: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JxlColorSpace {
    Rgb = 0,
    Gray,
    Xyb,
    Unknown,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JxlWhitePoint {
    D65 = 1,
    Custom = 2,
    E = 10,
    Dci = 11,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JxlPrimaries {
    SRgb = 1,
    Custom = 2,
    Rec2100 = 9,
    P3 = 11,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JxlTransferFunction {
    Rec709 = 1,
    Unknown = 2,
    Linear = 8,
    SRgb = 13,
    Pq = 16,
    Dci = 17,
    Hlg = 18,
    Gamma = 65535,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JxlRenderingIntent {
    Perceptual = 0,
    Relative,
    Saturation,
    Absolute,
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct JxlColorEncoding {
    pub color_space: JxlColorSpace,
    pub white_point: JxlWhitePoint,
    pub white_point_xy: [f64; 2usize],
    pub primaries: JxlPrimaries,
    pub primaries_red_xy: [f64; 2usize],
    pub primaries_green_xy: [f64; 2usize],
    pub primaries_blue_xy: [f64; 2usize],
    pub transfer_function: JxlTransferFunction,
    pub gamma: f64,
    pub rendering_intent: JxlRenderingIntent,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JxlOrientation {
    Identity = 1,
    FlipHorizontal = 2,
    Rotate180 = 3,
    FlipVertical = 4,
    Transpose = 5,
    Rotate90Cw = 6,
    AntiTranspose = 7,
    Rotate90Ccw = 8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JxlExtraChannelType {
    Alpha,
    Depth,
    SpotColor,
    SelectionMask,
    Black,
    Cfa,
    Thermal,
    Reserved0,
    Reserved1,
    Reserved2,
    Reserved3,
    Reserved4,
    Reserved5,
    Reserved6,
    Reserved7,
    Unknown,
    Optional,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JxlPreviewHeader {
    pub xsize: u32,
    pub ysize: u32,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JxlAnimationHeader {
    pub tps_numerator: u32,
    pub tps_denominator: u32,
    pub num_loops: u32,
    pub have_timecodes: JxlBool,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct JxlBasicInfo {
    pub have_container: JxlBool,
    pub xsize: u32,
    pub ysize: u32,
    pub bits_per_sample: u32,
    pub exponent_bits_per_sample: u32,
    pub intensity_target: f32,
    pub min_nits: f32,
    pub relative_to_max_display: JxlBool,
    pub linear_below: f32,
    pub uses_original_profile: JxlBool,
    pub have_preview: JxlBool,
    pub have_animation: JxlBool,
    pub orientation: JxlOrientation,
    pub num_color_channels: u32,
    pub num_extra_channels: u32,
    pub alpha_bits: u32,
    pub alpha_exponent_bits: u32,
    pub alpha_premultiplied: JxlBool,
    pub preview: JxlPreviewHeader,
    pub animation: JxlAnimationHeader,
    _padding: [u8; 108],
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct JxlExtraChannelInfo {
    pub type_: JxlExtraChannelType,
    pub bits_per_sample: u32,
    pub exponent_bits_per_sample: u32,
    pub dim_shift: u32,
    pub name_length: u32,
    pub alpha_associated: JxlBool,
    pub spot_color: [f32; 4usize],
    pub cfa_channel: u32,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JxlHeaderExtensions {
    pub extensions: u64,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JxlFrameHeader {
    pub duration: u32,
    pub timecode: u32,
    pub name_length: u32,
    pub is_last: JxlBool,
}

pub type JpegxlAllocFunc = unsafe extern "C" fn(opaque: *mut c_void, size: usize) -> *mut c_void;
pub type JpegxlFreeFunc = unsafe extern "C" fn(opaque: *mut c_void, address: *mut c_void);

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JxlMemoryManager {
    pub opaque: *mut c_void,
    pub alloc: JpegxlAllocFunc,
    pub free: JpegxlFreeFunc,
}

pub type JxlParallelRetCode = c_int;

pub type JxlParallelRunInit =
    unsafe extern "C" fn(jpegxl_opaque: *mut c_void, num_threads: usize) -> JxlParallelRetCode;

pub type JxlParallelRunFunction =
    unsafe extern "C" fn(jpegxl_opaque: *mut c_void, value: u32, thread_id: usize);

pub type JxlParallelRunner = unsafe extern "C" fn(
    runner_opaque: *mut c_void,
    jpegxl_opaque: *mut c_void,
    init: JxlParallelRunInit,
    func: JxlParallelRunFunction,
    start_range: u32,
    end_range: u32,
) -> JxlParallelRetCode;

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum JxlSignature {
    NotEnoughBytes = 0,
    Invalid = 1,
    Codestream = 2,
    Container = 3,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JxlColorProfileTarget {
    Original,
    Data,
}
