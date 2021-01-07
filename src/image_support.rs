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
use jpegxl_sys::JxlBasicInfo;

use crate::{JXLDecodeError, JXLDecoder};

#[cfg(any(feature = "with-image", test))]
impl From<JXLDecodeError> for image::ImageError {
    fn from(e: JXLDecodeError) -> Self {
        image::ImageError::Decoding(DecodingError::new(
            ImageFormatHint::Name("JPEGXL".to_string()),
            e,
        ))
    }
}

/// Jpeg XL representation for `image` crate
struct JxlImage {
    decoded_buffer: Vec<u8>,
    info: JxlBasicInfo,
}

impl JxlImage {
    pub fn new(buf: &[u8]) -> Result<Self, JXLDecodeError> {
        let mut decoder = JXLDecoder::default();
        let (info, decoded_buffer) = decoder.decode(&buf)?;
        Ok(Self {
            decoded_buffer,
            info,
        })
    }
}

impl ImageDecoder<'_> for JxlImage {
    type Reader = Self;

    fn dimensions(&self) -> (u32, u32) {
        return (self.info.xsize, self.info.ysize);
    }
    fn color_type(&self) -> ColorType {
        match self.info.bits_per_sample {
            8 => {
                if self.info.alpha_bits == 0 {
                    ColorType::Rgb8
                } else {
                    ColorType::Rgba8
                }
            }
            16 => {
                if self.info.alpha_bits == 0 {
                    ColorType::Rgb16
                } else {
                    ColorType::Rgba16
                }
            }
            _ => unreachable!("Encoder Not implemented!"),
        }
    }

    fn into_reader(self) -> image::ImageResult<Self::Reader> {
        Ok(self)
    }
}

impl std::io::Read for JxlImage {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.decoded_buffer.as_slice().read(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_support() -> Result<(), Box<dyn std::error::Error>> {
        let sample = std::fs::read("test/sample.jxl")?;
        let decoder = JxlImage::new(&sample)?;

        image::DynamicImage::from_decoder(decoder)?;

        Ok(())
    }
}
