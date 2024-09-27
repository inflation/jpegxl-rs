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

//! Data types for the JPEG XL API, for both encoding and decoding.

use std::ffi::c_char;

#[cfg(doc)]
use crate::metadata::codestream_header::JxlBasicInfo;

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

/// Data type for the sample values per channel per pixel.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JxlDataType {
    /// Use 32-bit single-precision floating point values, with range 0.0-1.0
    /// (within gamut, may go outside this range for wide color gamut). Floating
    /// point output, either [`JxlDataType::Float`] or [`JxlDataType::Float16`], is recommended
    /// for HDR and wide gamut images when color profile conversion is required.
    Float = 0,

    /// Use type `u8`. May clip wide color gamut data.
    Uint8 = 2,

    /// Use type `u16`. May clip wide color gamut data.
    Uint16 = 3,

    /// Use 16-bit IEEE 754 half-precision floating point values.
    Float16 = 5,
}

/// Ordering of multi-byte data.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JxlEndianness {
    /// Use the endianness of the system, either little endian or big endian,
    /// without forcing either specific endianness. Do not use if pixel data
    /// should be exported to a well defined format.
    Native = 0,
    /// Force little endian
    Little = 1,
    /// Force big endian
    Big = 2,
}

/// Data type for the sample values per channel per pixel for the output buffer
/// for pixels. This is not necessarily the same as the data type encoded in the
/// codestream. The channels are interleaved per pixel. The pixels are
/// organized row by row, left to right, top to bottom.
/// TODO: support different channel orders if needed (RGB, BGR, ...)
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct JxlPixelFormat {
    /// Amount of channels available in a pixel buffer.
    /// 1: single-channel data, e.g. grayscale or a single extra channel
    /// 2: single-channel + alpha
    /// 3: trichromatic, e.g. RGB
    /// 4: trichromatic + alpha
    /// TODO: this needs finetuning. It is not yet defined how the user
    /// chooses output color space. CMYK+alpha needs 5 channels.
    pub num_channels: u32,

    /// Data type of each channel.
    pub data_type: JxlDataType,

    /// Whether multi-byte data types are represented in big endian or little
    /// endian format. This applies to [`JxlDataType::Uint16`] and [`JxlDataType::Float`].
    pub endianness: JxlEndianness,

    /// Align scanlines to a multiple of align bytes, or 0 to require no
    /// alignment at all (which has the same effect as value 1)
    pub align: usize,
}

/// Settings for the interpretation of UINT input and output buffers.
/// (buffers using a FLOAT data type are not affected by this)
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JxlBitDepthType {
    /// This is the default setting, where the encoder expects the input pixels
    /// to use the full range of the pixel format data type (e.g. for UINT16, the
    /// input range is 0 .. 65535 and the value 65535 is mapped to 1.0 when
    /// converting to float), and the decoder uses the full range to output
    /// pixels. If the bit depth in the basic info is different from this, the
    /// encoder expects the values to be rescaled accordingly (e.g. multiplied by
    /// 65535/4095 for a 12-bit image using UINT16 input data type).
    FromPixelFormat = 0,

    /// If this setting is selected, the encoder expects the input pixels to be
    /// in the range defined by the [`JxlBasicInfo::bits_per_sample`] value (e.g.
    /// for 12-bit images using UINT16 input data types, the allowed range is
    /// 0 .. 4095 and the value 4095 is mapped to 1.0 when converting to float),
    /// and the decoder outputs pixels in this range.
    FromCodestream = 1,

    /// This setting can only be used in the decoder to select a custom range for
    /// pixel output.
    Custom = 2,
}

/// Data type for describing the interpretation of the input and output buffers
/// in terms of the range of allowed input and output pixel values.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct JxlBitDepth {
    /// Bit depth setting, see comment on [`JxlBitDepthType`]
    pub r#type: JxlBitDepthType,

    /// Custom bits per sample
    pub bits_per_sample: u32,

    /// Custom exponent bits per sample
    pub exponent_bits_per_sample: u32,
}

/// Data type holding the 4-character type name of an ISOBMFF box.
#[repr(transparent)]
pub struct JxlBoxType(pub [c_char; 4]);
