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

use byteorder::{ByteOrder, NativeEndian, BE, LE};
use half::f16;
use jpegxl_sys::{JxlDataType, JxlPixelFormat};

/// Endianness of the pixels
pub type Endianness = jpegxl_sys::JxlEndianness;

mod private {
    pub trait Sealed {}

    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for half::f16 {}
    impl Sealed for f32 {}
}

/// Pixel data type.
/// Currently `u8`, `u16`, `f16` and `f32` are supported.
pub trait PixelType: private::Sealed + Sized {
    /// Return the C const
    fn pixel_type() -> JxlDataType;

    /// Return number of bits per sample and exponential bits
    fn bits_per_sample() -> (u32, u32);

    /// Convert the data to the pixel type
    fn convert(data: &[u8], pixel_format: &JxlPixelFormat) -> Vec<Self>;
}

impl PixelType for u8 {
    fn pixel_type() -> JxlDataType {
        JxlDataType::Uint8
    }

    fn bits_per_sample() -> (u32, u32) {
        (8, 0)
    }

    fn convert(data: &[u8], _pixel_format: &JxlPixelFormat) -> Vec<Self> {
        data.to_vec()
    }
}

impl PixelType for u16 {
    fn pixel_type() -> JxlDataType {
        JxlDataType::Uint16
    }

    fn bits_per_sample() -> (u32, u32) {
        (16, 0)
    }

    fn convert(data: &[u8], pixel_format: &JxlPixelFormat) -> Vec<Self> {
        let mut buf = vec![u16::default(); data.len() / std::mem::size_of::<u16>()];
        match pixel_format.endianness {
            Endianness::Native => NativeEndian::read_u16_into(data, buf.as_mut_slice()),
            Endianness::Little => LE::read_u16_into(data, buf.as_mut_slice()),
            Endianness::Big => BE::read_u16_into(data, buf.as_mut_slice()),
        }
        buf
    }
}

impl PixelType for f32 {
    fn pixel_type() -> JxlDataType {
        JxlDataType::Float
    }

    // Float representation needs exponential bits
    fn bits_per_sample() -> (u32, u32) {
        (32, 8)
    }

    fn convert(data: &[u8], pixel_format: &JxlPixelFormat) -> Vec<Self> {
        let mut buf = vec![f32::default(); data.len() / std::mem::size_of::<f32>()];
        match pixel_format.endianness {
            Endianness::Native => NativeEndian::read_f32_into(data, buf.as_mut_slice()),
            Endianness::Little => LE::read_f32_into(data, buf.as_mut_slice()),
            Endianness::Big => BE::read_f32_into(data, buf.as_mut_slice()),
        }
        buf
    }
}

impl PixelType for f16 {
    fn pixel_type() -> JxlDataType {
        JxlDataType::Float16
    }

    fn bits_per_sample() -> (u32, u32) {
        (16, 5)
    }

    fn convert(data: &[u8], pixel_format: &JxlPixelFormat) -> Vec<Self> {
        data.chunks_exact(std::mem::size_of::<f16>())
            .map(|v| {
                f16::from_bits(match pixel_format.endianness {
                    Endianness::Native => NativeEndian::read_u16(v),
                    Endianness::Little => LE::read_u16(v),
                    Endianness::Big => BE::read_u16(v),
                })
            })
            .collect()
    }
}
