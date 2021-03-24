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

use crate::{
    common::PixelType,
    decoder_builder,
    errors::{DecodeError, EncodeError},
    DecoderResult, DecoderResultInfo,
};

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
/// let decoder: JxlImageDecoder<u16> = JxlImageDecoder::new(&sample)?;
/// let img = image::DynamicImage::from_decoder(decoder)?;       
/// # Ok(()) };
/// ```
pub struct JxlImageDecoder<T: PixelType> {
    info: DecoderResultInfo,
    data: Vec<T>,
}

impl<T: PixelType> JxlImageDecoder<T> {
    /// Create a new JPEG XL Decoder.
    /// # Errors
    /// Return an [`image::ImageError`] with wrapped [`DecodeError`]
    pub fn new(input: &[u8]) -> ImageResult<JxlImageDecoder<T>> {
        let mut dec = decoder_builder().build()?;
        // TODO: Stream decoding
        let DecoderResult { info, data, .. } = dec.decode(&input)?;

        let decoder = JxlImageDecoder { info, data };
        Ok(decoder)
    }
}

impl<'a> ImageDecoder<'a> for JxlImageDecoder<u8> {
    type Reader = std::io::Cursor<Vec<u8>>;

    fn color_type(&self) -> ColorType {
        if self.info.has_alpha {
            ColorType::Rgba8
        } else {
            ColorType::Rgb8
        }
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.info.width, self.info.height)
    }

    fn into_reader(self) -> image::ImageResult<Self::Reader> {
        Ok(std::io::Cursor::new(self.data))
    }
}

impl<'a> ImageDecoder<'a> for JxlImageDecoder<u16> {
    type Reader = std::io::Cursor<Vec<u8>>;

    fn color_type(&self) -> ColorType {
        if self.info.has_alpha {
            ColorType::Rgba16
        } else {
            ColorType::Rgb16
        }
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.info.width, self.info.height)
    }

    fn into_reader(self) -> image::ImageResult<Self::Reader> {
        let mut v = std::mem::ManuallyDrop::new(self.data);

        let vec_u8 = unsafe {
            Vec::from_raw_parts(
                v.as_mut_ptr().cast(),
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
        let decoder: JxlImageDecoder<u16> = JxlImageDecoder::new(&sample)?;

        let img = image::DynamicImage::from_decoder(decoder)?;
        let sample_png = image::io::Reader::open("test/sample.png")?.decode()?;

        assert_eq!(img.as_rgb8(), sample_png.as_rgb8());

        Ok(())
    }
}
