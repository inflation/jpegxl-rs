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
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlDataType {
    Float = 0,
    Uint8 = 2,
    Uint16 = 3,
    Float16 = 5,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlEndianness {
    Native = 0,
    Little,
    Big,
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct JxlPixelFormat {
    pub num_channels: u32,
    pub data_type: JxlDataType,
    pub endianness: JxlEndianness,
    pub align: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlBitDepthType {
    BitDepthFromPixelFormat = 0,
    BitDepthFromCodestream = 1,
    BitDepthCustom = 2,
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct JxlBitDepth {
    type_: JxlBitDepthType,
    bits_per_sample: u32,
    exponent_bits_per_sample: u32,
}

pub type JxlBoxType = [c_char; 4];
