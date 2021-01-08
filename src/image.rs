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

//! `image` crate integration

use image::{
    error::{DecodingError, EncodingError, ImageFormatHint},
    ColorType, ImageDecoder, ImageResult,
};

use jpegxl_sys::{JxlDataType_JXL_TYPE_UINT16, JxlDataType_JXL_TYPE_UINT8};

use crate::{common::*, decoder_builder, error::*, BasicInfo, JXLDecoder};

// Error conversion
impl From<DecodeError> for image::ImageError {
    fn from(e: DecodeError) -> Self {
        image::ImageError::Decoding(DecodingError::new(
            ImageFormatHint::Name("JPEGXL".to_string()),
            e,
        ))
    }
}

impl From<EncodeError> for image::ImageError {
    fn from(e: EncodeError) -> Self {
        image::ImageError::Encoding(EncodingError::new(
            ImageFormatHint::Name("JPEGXL".to_string()),
            e,
        ))
    }
}

/// JPEG XL decoder representation for `image` crate.<br />
/// Note: Since `image` only supports 8-bit and 16-bit pixel depth,
/// you can only create `u8` and `u16` buffer.
/// # Example
/// ```
/// # || -> Result<(), Box<dyn std::error::Error>> {
/// # use jpegxl_rs::image::*;
/// let sample = std::fs::read("test/sample.jxl")?;
/// let decoder: JXLImageDecoder<u16> = JXLImageDecoder::new(&sample)?;
/// let img = image::DynamicImage::from_decoder(decoder)?;       
/// # Ok(()) };
/// ```
pub struct JXLImageDecoder<T: PixelType> {
    info: BasicInfo,
    buffer: Vec<T>,
}

impl<T: PixelType> JXLImageDecoder<T> {
    /// Create a new JPEG XL Decoder.
    pub fn new(input: &[u8]) -> ImageResult<JXLImageDecoder<T>> {
        let mut dec: JXLDecoder<T> = decoder_builder().build();
        // TODO: Stream decoding
        let (info, buffer) = dec.decode(input)?;

        let decoder = JXLImageDecoder { info, buffer };
        Ok(decoder)
    }
}

impl<'a, T: PixelType> ImageDecoder<'a> for JXLImageDecoder<T> {
    type Reader = std::io::Cursor<Vec<u8>>;

    fn color_type(&self) -> ColorType {
        match (T::pixel_type(), self.info.num_extra_channels) {
            (JxlDataType_JXL_TYPE_UINT8, 0) => ColorType::Rgb8,
            (JxlDataType_JXL_TYPE_UINT16, 0) => ColorType::Rgb16,
            (JxlDataType_JXL_TYPE_UINT8, _) => ColorType::Rgba8,
            (JxlDataType_JXL_TYPE_UINT16, _) => ColorType::Rgba16,
            _ => unimplemented!(),
        }
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.info.xsize, self.info.ysize)
    }

    fn into_reader(self) -> image::ImageResult<Self::Reader> {
        let mut v = std::mem::ManuallyDrop::new(self.buffer);

        let vec_u8 = unsafe {
            Vec::from_raw_parts(
                v.as_mut_ptr() as *mut u8,
                v.len() * std::mem::size_of::<u16>(),
                v.capacity(),
            )
        };
        Ok(std::io::Cursor::new(vec_u8))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_support() -> Result<(), Box<dyn std::error::Error>> {
        let sample = std::fs::read("test/sample.jxl")?;
        let decoder: JXLImageDecoder<u16> = JXLImageDecoder::new(&sample)?;

        let img = image::DynamicImage::from_decoder(decoder)?;
        let sample_png = image::io::Reader::open("test/sample.png")?.decode()?;

        assert_eq!(img.as_rgb8(), sample_png.as_rgb8());

        Ok(())
    }
}
