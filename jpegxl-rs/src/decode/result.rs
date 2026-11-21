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

use half::f16;
use jpegxl_sys::common::types::{JxlDataType, JxlPixelFormat};

use super::Orientation;
use crate::common::PixelType;

/// Result of decoding
#[derive(Debug)]
pub struct Metadata {
    /// Width of the returned image, or of its final layer when coalescing is disabled.
    pub width: u32,
    /// Height of the returned image, or of its final layer when coalescing is disabled.
    pub height: u32,
    /// Upper bound on the intensity level present in the image in nits
    pub intensity_target: f32,
    /// Lower bound on the intensity level present in the image
    pub min_nits: f32,
    /// Orientation
    pub orientation: Orientation,
    /// Number of color channels per pixel _without_ alpha channel, from metadata
    pub num_color_channels: u32,
    /// Whether the image has an alpha channel, from metadata
    pub has_alpha_channel: bool,
    /// Intrinsic width of the image.
    /// Applications are advised to resample the decoded image to the intrinsic dimensions
    pub intrinsic_width: u32,
    /// Intrinsic height of the image.
    /// Applications are advised to resample the decoded image to the intrinsic dimensions
    pub intrinsic_height: u32,
    /// ICC profile
    pub icc_profile: Option<Vec<u8>>,
}

/// Pixels returned from the decoder
#[derive(Debug)]
pub enum Pixels {
    /// `f32` pixels
    Float(Vec<f32>),
    /// `u8` pixels
    Uint8(Vec<u8>),
    /// `u16` pixels
    Uint16(Vec<u16>),
    /// `f16` pixels
    Float16(Vec<f16>),
}

impl Pixels {
    pub(crate) fn new(data: Vec<u8>, pixel_format: &JxlPixelFormat) -> Self {
        match pixel_format.data_type {
            JxlDataType::Float => Self::Float(f32::convert(&data, pixel_format)),
            JxlDataType::Uint8 => Self::Uint8(data),
            JxlDataType::Uint16 => Self::Uint16(u16::convert(&data, pixel_format)),
            JxlDataType::Float16 => Self::Float16(f16::convert(&data, pixel_format)),
        }
    }
}

/// Raw pixels with the format they were decoded to
#[derive(Debug)]
pub struct Image {
    /// Width of this frame, preview, or extra channel.
    pub width: u32,
    /// Height of this frame, preview, or extra channel.
    pub height: u32,
    /// Pixel format of `data`
    pub format: JxlPixelFormat,
    /// Pixel bytes. Rows are padded to the alignment of `format`
    pub data: Vec<u8>,
}

impl Image {
    /// Convert the bytes to tightly packed [`Pixels`], discarding scanline padding.
    #[must_use]
    pub fn into_pixels(mut self) -> Pixels {
        let bytes_per_sample = match self.format.data_type {
            JxlDataType::Uint8 => 1,
            JxlDataType::Uint16 | JxlDataType::Float16 => 2,
            JxlDataType::Float => 4,
        };
        let row_bytes = self.width as usize * self.format.num_channels as usize * bytes_per_sample;
        let height = self.height as usize;
        if self.format.align > 1 && height > 1 {
            let stride = row_bytes.next_multiple_of(self.format.align);
            if stride != row_bytes {
                for row in 1..height {
                    let start = row * stride;
                    self.data
                        .copy_within(start..start + row_bytes, row * row_bytes);
                }
            }
        }
        self.data.truncate(height * row_bytes);
        Pixels::new(self.data, &self.format)
    }
}

/// Reconstruction result
pub enum Data {
    /// JPEG  
    Jpeg(Vec<u8>),
    /// Pixels
    Pixels(Pixels),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_pixels_ignores_row_padding_and_spare_buffer_space() {
        for (width, height, data, expected) in [
            (1, 2, vec![0, 1, 255, 255, 1, 0, 255, 255], vec![1, 256]),
            (2, 2, vec![0, 1, 1, 0, 0, 2, 2, 0], vec![1, 256, 2, 512]),
            (1, 1, vec![0, 1, 255, 255], vec![1]),
        ] {
            let image = Image {
                width,
                height,
                format: JxlPixelFormat {
                    num_channels: 1,
                    data_type: JxlDataType::Uint16,
                    endianness: crate::Endianness::Big,
                    align: 4,
                },
                data,
            };
            assert!(matches!(image.into_pixels(), Pixels::Uint16(values) if values == expected));
        }
    }
}
