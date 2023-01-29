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

use std::ffi::c_char;

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum JxlBool {
    True = 1,
    False = 0,
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
    Uint8 = 2,
    Uint16 = 3,
    Float16 = 5,
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JxlBitDepthType {
    BitDepthFromPixelFormat = 0,
    BitDepthFromCodestream = 1,
    BitDepthCustom = 2,
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JxlBitDepth {
    type_: JxlBitDepthType,
    bits_per_sample: u32,
    exponent_bits_per_sample: u32,
}

pub type JxlBoxType = [c_char; 4];

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JxlProgressiveDetail {
    Frames = 0,
    DC = 1,
    LastPasses = 2,
    Passes = 3,
    DCProgressive = 4,
    DCGroups = 5,
    Groups = 6,
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
#[derive(Clone, Debug)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JxlColorProfileTarget {
    Original,
    Data,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jxl_bool() {
        assert_eq!(JxlBool::from(true), JxlBool::True);
        assert_eq!(JxlBool::from(false), JxlBool::False);
    }
}
