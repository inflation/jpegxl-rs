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

#![cfg_attr(docsrs, doc(cfg(feature = "image")))]

//! `image` crate integration

use std::mem::MaybeUninit;

use image::{DynamicImage, ImageBuffer};
use jpegxl_sys::JxlDataType;

use crate::{
    decode::{to_f32, to_u16, JxlDecoder, Metadata},
    DecodeError,
};

/// Extension trait for [`JxlDecoder`]
pub trait ToDynamic {
    /// Decode the JPEG XL image to a [`DynamicImage`]
    ///
    /// # Errors
    /// Return a [`DecodeError`] when internal decoding fails. Return `Ok(None)` when the image is not
    /// representable as a [`DynamicImage`]
    fn decode_to_dynamic_image(&self, data: &[u8]) -> Result<Option<DynamicImage>, DecodeError>;
}

impl<'pr, 'mm> ToDynamic for JxlDecoder<'pr, 'mm> {
    fn decode_to_dynamic_image(&self, data: &[u8]) -> Result<Option<DynamicImage>, DecodeError> {
        let mut buffer = vec![];
        let mut pixel_format = MaybeUninit::uninit();
        let Metadata {
            width,
            height,
            num_color_channels,
            has_alpha_channel,
            ..
        } = self.decode_internal(
            data,
            None,
            self.icc_profile,
            None,
            (pixel_format.as_mut_ptr(), &mut buffer),
        )?;

        let pixel_format = unsafe { pixel_format.assume_init() };
        Ok(match pixel_format.data_type {
            JxlDataType::Float => {
                if num_color_channels == 3 {
                    if has_alpha_channel {
                        ImageBuffer::from_raw(width, height, to_f32(&buffer, &pixel_format))
                            .map(DynamicImage::ImageRgba32F)
                    } else {
                        ImageBuffer::from_raw(width, height, to_f32(&buffer, &pixel_format))
                            .map(DynamicImage::ImageRgb32F)
                    }
                } else {
                    None
                }
            }
            JxlDataType::Uint8 => {
                if num_color_channels == 1 {
                    if has_alpha_channel {
                        ImageBuffer::from_raw(width, height, buffer).map(DynamicImage::ImageLumaA8)
                    } else {
                        ImageBuffer::from_raw(width, height, buffer).map(DynamicImage::ImageLuma8)
                    }
                } else if num_color_channels == 3 {
                    if has_alpha_channel {
                        ImageBuffer::from_raw(width, height, buffer).map(DynamicImage::ImageRgba8)
                    } else {
                        ImageBuffer::from_raw(width, height, buffer).map(DynamicImage::ImageRgb8)
                    }
                } else {
                    None
                }
            }
            JxlDataType::Uint16 => {
                if num_color_channels == 1 {
                    if has_alpha_channel {
                        ImageBuffer::from_raw(width, height, to_u16(&buffer, &pixel_format))
                            .map(DynamicImage::ImageLumaA16)
                    } else {
                        ImageBuffer::from_raw(width, height, to_u16(&buffer, &pixel_format))
                            .map(DynamicImage::ImageLuma16)
                    }
                } else if num_color_channels == 3 {
                    if has_alpha_channel {
                        ImageBuffer::from_raw(width, height, to_u16(&buffer, &pixel_format))
                            .map(DynamicImage::ImageRgba16)
                    } else {
                        ImageBuffer::from_raw(width, height, to_u16(&buffer, &pixel_format))
                            .map(DynamicImage::ImageRgb16)
                    }
                } else {
                    None
                }
            }
            JxlDataType::Float16 => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        decode::PixelFormat,
        decoder_builder,
        tests::{SAMPLE_JXL, SAMPLE_JXL_GRAY, SAMPLE_PNG},
        ThreadsRunner,
    };

    use super::*;

    #[test]
    fn simple() {
        let parallel_runner = ThreadsRunner::default();
        let decoder = decoder_builder()
            .parallel_runner(&parallel_runner)
            .build()
            .unwrap();

        let img = decoder
            .decode_to_dynamic_image(SAMPLE_JXL)
            .unwrap()
            .expect("Failed to create DynamicImage");
        let sample_png =
            image::load_from_memory_with_format(SAMPLE_PNG, image::ImageFormat::Png).unwrap();

        assert_eq!(img.to_rgba16(), sample_png.to_rgba16());
    }

    #[test]
    fn channels() {
        let parallel_runner = ThreadsRunner::default();
        let mut decoder = decoder_builder()
            .parallel_runner(&parallel_runner)
            .pixel_format(PixelFormat {
                num_channels: 1,
                ..PixelFormat::default()
            })
            .build()
            .unwrap();
        decoder
            .decode_to_dynamic_image(SAMPLE_JXL_GRAY)
            .unwrap()
            .unwrap();

        decoder.pixel_format = Some(PixelFormat {
            num_channels: 2,
            ..PixelFormat::default()
        });
        decoder
            .decode_to_dynamic_image(SAMPLE_JXL_GRAY)
            .unwrap()
            .unwrap();

        decoder.pixel_format = Some(PixelFormat {
            num_channels: 3,
            ..PixelFormat::default()
        });
        decoder
            .decode_to_dynamic_image(SAMPLE_JXL_GRAY)
            .unwrap()
            .unwrap();
    }
}
