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
use jpegxl_sys::{JxlDataType, JxlPixelFormat};

use super::Orientation;
use crate::common::PixelType;

/// Result of decoding
#[derive(Debug)]
pub struct Metadata {
    /// Width of the image
    pub width: u32,
    /// Height of the image
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
    #[cfg_attr(coverage_nightly, no_coverage)]
    fn test_derive() {
        println!(
            "{:?}",
            Metadata {
                width: 0,
                height: 0,
                intensity_target: 0.0,
                min_nits: 0.0,
                orientation: Orientation::Identity,
                num_color_channels: 0,
                has_alpha_channel: false,
                intrinsic_width: 0,
                intrinsic_height: 0,
                icc_profile: None,
            }
        );

        println!("{:?}", Pixels::Float(vec![]));
    }
}
