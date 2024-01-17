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
use jpegxl_sys::types::{JxlDataType, JxlPixelFormat};

use crate::{
    common::PixelType,
    decode::{JxlDecoder, Metadata},
    DecodeError,
};

/// Extension trait for [`JxlDecoder`]
pub trait ToDynamic {
    /// Decode the JPEG XL image to a [`DynamicImage`]
    ///
    /// # Errors
    /// Return a [`DecodeError`] when internal decoding fails.
    /// Return `Ok(None)` when the image is not representable as a [`DynamicImage`]
    fn decode_to_image(&self, data: &[u8]) -> Result<Option<DynamicImage>, DecodeError>;

    /// Decode the JPEG XL image to a [`DynamicImage`] with a specific pixel type
    ///
    /// # Errors
    /// Return a [`DecodeError`] when internal decoding fails.
    /// Return `Ok(None)` when the image is not representable as a [`DynamicImage`]
    fn decode_to_image_with<T: PixelType>(
        &self,
        data: &[u8],
    ) -> Result<Option<DynamicImage>, DecodeError>;
}

impl<'pr, 'mm> ToDynamic for JxlDecoder<'pr, 'mm> {
    fn decode_to_image(&self, data: &[u8]) -> Result<Option<DynamicImage>, DecodeError> {
        let mut buffer = vec![];
        let mut pixel_format = MaybeUninit::uninit();
        let metadata = self.decode_internal(
            data,
            None,
            false,
            None,
            pixel_format.as_mut_ptr(),
            &mut buffer,
        )?;

        let pixel_format = unsafe { pixel_format.assume_init() };
        Ok(to_image(metadata, &pixel_format, buffer))
    }

    fn decode_to_image_with<T: PixelType>(
        &self,
        data: &[u8],
    ) -> Result<Option<DynamicImage>, DecodeError> {
        let mut buffer = vec![];
        let mut pixel_format = MaybeUninit::uninit();
        let metadata = self.decode_internal(
            data,
            Some(T::pixel_type()),
            false,
            None,
            pixel_format.as_mut_ptr(),
            &mut buffer,
        )?;

        let pixel_format = unsafe { pixel_format.assume_init() };
        Ok(to_image(metadata, &pixel_format, buffer))
    }
}

fn to_image(
    Metadata { width, height, .. }: Metadata,
    pixel_format: &JxlPixelFormat,
    buffer: Vec<u8>,
) -> Option<DynamicImage> {
    match (pixel_format.data_type, pixel_format.num_channels) {
        (JxlDataType::Float, 3) => {
            ImageBuffer::from_raw(width, height, f32::convert(&buffer, pixel_format))
                .map(DynamicImage::ImageRgb32F)
        }
        (JxlDataType::Float, 4) => {
            ImageBuffer::from_raw(width, height, f32::convert(&buffer, pixel_format))
                .map(DynamicImage::ImageRgba32F)
        }
        (JxlDataType::Uint8, 1) => {
            ImageBuffer::from_raw(width, height, buffer).map(DynamicImage::ImageLuma8)
        }
        (JxlDataType::Uint8, 2) => {
            ImageBuffer::from_raw(width, height, buffer).map(DynamicImage::ImageLumaA8)
        }
        (JxlDataType::Uint8, 3) => {
            ImageBuffer::from_raw(width, height, buffer).map(DynamicImage::ImageRgb8)
        }
        (JxlDataType::Uint8, 4) => {
            ImageBuffer::from_raw(width, height, buffer).map(DynamicImage::ImageRgba8)
        }
        (JxlDataType::Uint16, 1) => {
            ImageBuffer::from_raw(width, height, u16::convert(&buffer, pixel_format))
                .map(DynamicImage::ImageLuma16)
        }
        (JxlDataType::Uint16, 2) => {
            ImageBuffer::from_raw(width, height, u16::convert(&buffer, pixel_format))
                .map(DynamicImage::ImageLumaA16)
        }
        (JxlDataType::Uint16, 3) => {
            ImageBuffer::from_raw(width, height, u16::convert(&buffer, pixel_format))
                .map(DynamicImage::ImageRgb16)
        }
        (JxlDataType::Uint16, 4) => {
            ImageBuffer::from_raw(width, height, u16::convert(&buffer, pixel_format))
                .map(DynamicImage::ImageRgba16)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use half::f16;
    use testresult::TestResult;

    use crate::{
        decode::PixelFormat,
        decoder_builder,
        tests::{SAMPLE_JXL, SAMPLE_JXL_GRAY, SAMPLE_PNG},
        ThreadsRunner,
    };

    use super::*;

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn invalid() -> TestResult {
        let decoder = decoder_builder().build()?;
        assert!(decoder.decode_to_image(&[]).is_err());
        assert!(decoder.decode_to_image_with::<f32>(&[]).is_err());
        Ok(())
    }

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn simple() -> TestResult {
        let parallel_runner = ThreadsRunner::default();
        let decoder = decoder_builder()
            .parallel_runner(&parallel_runner)
            .build()?;

        let img = decoder
            .decode_to_image(SAMPLE_JXL)?
            .expect("Failed to create DynamicImage");
        let sample_png = image::load_from_memory_with_format(SAMPLE_PNG, image::ImageFormat::Png)?;
        assert_eq!(img.to_rgba16(), sample_png.to_rgba16());

        Ok(())
    }

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn pixel_type() -> TestResult {
        let parallel_runner = ThreadsRunner::default();
        let mut decoder = decoder_builder()
            .parallel_runner(&parallel_runner)
            .build()?;
        assert!(decoder.decode_to_image_with::<f16>(SAMPLE_JXL)?.is_none());

        decoder.pixel_format = Some(PixelFormat {
            num_channels: 1,
            ..PixelFormat::default()
        });
        decoder
            .decode_to_image_with::<u8>(SAMPLE_JXL_GRAY)?
            .unwrap();
        decoder
            .decode_to_image_with::<u16>(SAMPLE_JXL_GRAY)?
            .unwrap();
        assert!(decoder
            .decode_to_image_with::<f32>(SAMPLE_JXL_GRAY)?
            .is_none());

        decoder.pixel_format = Some(PixelFormat {
            num_channels: 2,
            ..PixelFormat::default()
        });
        decoder
            .decode_to_image_with::<u8>(SAMPLE_JXL_GRAY)?
            .unwrap();
        decoder
            .decode_to_image_with::<u16>(SAMPLE_JXL_GRAY)?
            .unwrap();
        assert!(decoder
            .decode_to_image_with::<f32>(SAMPLE_JXL_GRAY)?
            .is_none());

        decoder.pixel_format = Some(PixelFormat {
            num_channels: 3,
            ..PixelFormat::default()
        });
        decoder.decode_to_image_with::<u8>(SAMPLE_JXL)?.unwrap();
        decoder.decode_to_image_with::<u16>(SAMPLE_JXL)?.unwrap();
        decoder.decode_to_image_with::<f32>(SAMPLE_JXL)?.unwrap();

        decoder.pixel_format = Some(PixelFormat {
            num_channels: 4,
            ..PixelFormat::default()
        });
        decoder.decode_to_image_with::<u8>(SAMPLE_JXL)?.unwrap();
        decoder.decode_to_image_with::<u16>(SAMPLE_JXL)?.unwrap();
        decoder.decode_to_image_with::<f32>(SAMPLE_JXL)?.unwrap();

        Ok(())
    }
}
