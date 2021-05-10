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

//! Common types used across the crate

use jpegxl_sys::JxlDataType;

/// Endianness of the pixels
pub type Endianness = jpegxl_sys::JxlEndianness;

/// Pixel data type.
/// Currently `u8`, `u16` and `f32`(partial) are supported.
/// Notes: The encoder does not support f32 with alpha channel
pub trait PixelType: Clone + Default + 'static {
    /// Return the c const
    fn pixel_type() -> JxlDataType;

    /// Return number of bits per sample and exponential bits
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    fn bits_per_sample() -> (u32, u32) {
        ((std::mem::size_of::<Self>() * 8) as u32, 0)
    }
}

impl PixelType for u8 {
    fn pixel_type() -> JxlDataType {
        JxlDataType::Uint8
    }
}

impl PixelType for u16 {
    fn pixel_type() -> JxlDataType {
        JxlDataType::Uint16
    }
}

// TODO: Upstream decoder does not support the type though listed as valid
// impl PixelType for u32 {
//     fn pixel_type() -> JxlDataType {
//         JxlDataType::Uint32
//     }
// }

// TODO: Upstream encoder does not support alpha channel
impl PixelType for f32 {
    fn pixel_type() -> JxlDataType {
        JxlDataType::Float
    }

    /// Float representation needs exponential bits
    fn bits_per_sample() -> (u32, u32) {
        (32, 8)
    }
}
