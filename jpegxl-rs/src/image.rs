/*
 * This file is part of jpegxl-rs.
 *
 * jpegxl-rs is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * jpegxl-rs is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with jpegxl-rs.  If not, see <https://www.gnu.org/licenses/>.
 */

//! `image` crate integration

use image::{DynamicImage, ImageBuffer};
use jpegxl_sys::common::types::JxlDataType;

use crate::common::PixelType;
use crate::decode::{Image, JxlDecoder};
use crate::DecodeError;

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

impl ToDynamic for JxlDecoder<'_, '_> {
    fn decode_to_image(&self, data: &[u8]) -> Result<Option<DynamicImage>, DecodeError> {
        let (_, image, _) = self.decode_internal(data, None, false, false)?;
        Ok(image.and_then(to_image))
    }

    fn decode_to_image_with<T: PixelType>(
        &self,
        data: &[u8],
    ) -> Result<Option<DynamicImage>, DecodeError> {
        let (_, image, _) = self.decode_internal(data, Some(T::pixel_type()), false, false)?;
        Ok(image.and_then(to_image))
    }
}

fn to_image(
    Image {
        width,
        height,
        format: pixel_format,
        data: buffer,
    }: Image,
) -> Option<DynamicImage> {
    match (pixel_format.data_type, pixel_format.num_channels) {
        (JxlDataType::Float, 3) => {
            ImageBuffer::from_raw(width, height, f32::convert(&buffer, &pixel_format))
                .map(DynamicImage::ImageRgb32F)
        }
        (JxlDataType::Float, 4) => {
            ImageBuffer::from_raw(width, height, f32::convert(&buffer, &pixel_format))
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
            ImageBuffer::from_raw(width, height, u16::convert(&buffer, &pixel_format))
                .map(DynamicImage::ImageLuma16)
        }
        (JxlDataType::Uint16, 2) => {
            ImageBuffer::from_raw(width, height, u16::convert(&buffer, &pixel_format))
                .map(DynamicImage::ImageLumaA16)
        }
        (JxlDataType::Uint16, 3) => {
            ImageBuffer::from_raw(width, height, u16::convert(&buffer, &pixel_format))
                .map(DynamicImage::ImageRgb16)
        }
        (JxlDataType::Uint16, 4) => {
            ImageBuffer::from_raw(width, height, u16::convert(&buffer, &pixel_format))
                .map(DynamicImage::ImageRgba16)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::decode::PixelFormat;
    use half::f16;
    use pretty_assertions::assert_eq;
    use testresult::TestResult;

    use super::*;
    use crate::tests::{SAMPLE_JXL, SAMPLE_JXL_GRAY, SAMPLE_PNG};
    use crate::{decoder_builder, ThreadsRunner};

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
            .pixel_format(PixelFormat {
                align: 4096,
                ..PixelFormat::default()
            })
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
        use image::ColorType::{La16, La8, Rgb16, Rgb32F, Rgb8, Rgba16, Rgba32F, Rgba8, L16, L8};

        let mut decoder = decoder_builder().build()?;
        assert!(decoder.decode_to_image_with::<f16>(SAMPLE_JXL)?.is_none());
        for (num_channels, sample, u8_color, u16_color, f32_color) in [
            (1, SAMPLE_JXL_GRAY, L8, L16, None),
            (2, SAMPLE_JXL_GRAY, La8, La16, None),
            (3, SAMPLE_JXL, Rgb8, Rgb16, Some(Rgb32F)),
            (4, SAMPLE_JXL, Rgba8, Rgba16, Some(Rgba32F)),
        ] {
            decoder.pixel_format = Some(PixelFormat {
                num_channels,
                ..PixelFormat::default()
            });
            assert_eq!(
                decoder.decode_to_image_with::<u8>(sample)?.unwrap().color(),
                u8_color
            );
            assert_eq!(
                decoder
                    .decode_to_image_with::<u16>(sample)?
                    .unwrap()
                    .color(),
                u16_color
            );
            assert_eq!(
                decoder
                    .decode_to_image_with::<f32>(sample)?
                    .map(|image| image.color()),
                f32_color
            );
        }

        Ok(())
    }
}
