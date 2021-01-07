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

use image::error::{DecodingError, ImageFormatHint};
use image::ColorType;
use image::ImageDecoder;

use crate::{JXLBasicInfo, JXLDecodeError, JXLDecoder, PixelType};

#[cfg(feature = "with-image")]
impl From<JXLDecodeError> for image::ImageError {
    fn from(e: JXLDecodeError) -> Self {
        image::ImageError::Decoding(DecodingError::new(
            ImageFormatHint::Name("JPEGXL".to_string()),
            e,
        ))
    }
}

/// JPEG XL representation for `image` crate
/// Note: Since `image` only supports 8-bit and 16-bit pixel depth,
/// you can only create `u8` and `u16` buffer.
pub struct JXLImage<T: PixelType> {
    buffer: Vec<T>,
    info: JXLBasicInfo,
}

impl<T: PixelType> JXLImage<T> {
    /// Create a new JPEG XL image. Note that this will initialize underlying decoder everytime.
    /// If you need to decode multiple files, use the decoder directly.
    pub fn new(buf: &[u8]) -> Result<Self, JXLDecodeError> {
        let mut decoder = JXLDecoder::default();
        let (info, buffer) = decoder.decode(&buf)?;
        Ok(Self { buffer, info })
    }
}

impl ImageDecoder<'_> for JXLImage<u8> {
    type Reader = Self;

    fn color_type(&self) -> ColorType {
        if self.info.alpha_bits == 0 {
            ColorType::Rgb8
        } else {
            ColorType::Rgba8
        }
    }

    fn dimensions(&self) -> (u32, u32) {
        return (self.info.xsize, self.info.ysize);
    }

    fn into_reader(self) -> image::ImageResult<Self::Reader> {
        Ok(self)
    }
}

impl ImageDecoder<'_> for JXLImage<u16> {
    type Reader = Self;

    fn color_type(&self) -> ColorType {
        if self.info.alpha_bits == 0 {
            ColorType::Rgb16
        } else {
            ColorType::Rgba16
        }
    }

    fn dimensions(&self) -> (u32, u32) {
        return (self.info.xsize, self.info.ysize);
    }

    fn into_reader(self) -> image::ImageResult<Self::Reader> {
        Ok(self)
    }
}

impl std::io::Read for JXLImage<u8> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.buffer.as_slice().read(buf)
    }
}

impl std::io::Read for JXLImage<u16> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        unsafe { std::mem::transmute::<&[u16], &[u8]>(self.buffer.as_slice()).read(buf) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_support() -> Result<(), Box<dyn std::error::Error>> {
        let sample = std::fs::read("test/sample.jxl")?;
        let decoder: JXLImage<u16> = JXLImage::new(&sample)?;

        image::DynamicImage::from_decoder(decoder)?;

        Ok(())
    }
}
