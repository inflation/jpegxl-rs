use image::error::{DecodingError, ImageFormatHint};
use image::ColorType;
use image::ImageDecoder;

use crate::{JxlDecoder, JxlError};

#[cfg(any(feature = "with-image", test))]
impl From<JxlError> for image::ImageError {
    fn from(e: JxlError) -> Self {
        image::ImageError::Decoding(DecodingError::new(
            ImageFormatHint::Name("JPEGXL".to_string()),
            e,
        ))
    }
}

/// Jpeg XL representation for `image` crate
struct JxlImage {
    decoded_buffer: Vec<u8>,
    decoder: JxlDecoder,
}

impl JxlImage {
    pub fn new(buf: impl AsRef<[u8]>) -> Result<Self, JxlError> {
        let mut decoder =
            JxlDecoder::new_with_default().ok_or(JxlError::CannotCreateDecoder)?;
        let decoded_buffer = decoder.decode(&buf)?;
        Ok(Self {
            decoded_buffer,
            decoder,
        })
    }
}

impl ImageDecoder<'_> for JxlImage {
    type Reader = Self;

    fn dimensions(&self) -> (u32, u32) {
        let basic_info = self.decoder.basic_info.unwrap();
        return (basic_info.xsize, basic_info.ysize);
    }
    fn color_type(&self) -> ColorType {
        let basic_info = self.decoder.basic_info.unwrap();
        match basic_info.bits_per_sample {
            8 => {
                if basic_info.alpha_bits == 0 {
                    ColorType::Rgb8
                } else {
                    ColorType::Rgba8
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

        let image = image::DynamicImage::from_decoder(decoder)?;
        image.save("sample.png")?;
        Ok(())
    }
}
