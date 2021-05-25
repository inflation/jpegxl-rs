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

#![cfg_attr(docsrs, doc(cfg(feature = "image-support")))]

//! `image` crate integration

use image::{DynamicImage, ImageBuffer};
use paste::paste;

use crate::decode::DecoderResult;

/// Extension trait for [`DecoderResult`]
pub trait ToDynamic {
    /// Convert the decoded result to a [`DynamicImage`]
    fn into_dynamic_image(self) -> Option<DynamicImage>;
}

macro_rules! to_dynamic {
    ($self:ident, $x:ident) => {
        ImageBuffer::from_raw($self.info.width, $self.info.height, $self.data).map(DynamicImage::$x)
    };
}

macro_rules! impl_to_dynamic_for_bytes {
    ($($b:expr),*) => {
        $(paste! {
            impl ToDynamic for DecoderResult<[<u $b>]> {
                fn into_dynamic_image(self) -> Option<DynamicImage> {
                    match self.info.num_channels {
                        1 => to_dynamic!(self, [<ImageLuma $b>]),
                        2 => to_dynamic!(self, [<ImageLumaA $b>]),
                        3 => to_dynamic!(self, [<ImageRgb $b>]),
                        4 => to_dynamic!(self, [<ImageRgba $b>]),
                        _ => None,
                    }
                }
            }
        })*
    };
}

impl_to_dynamic_for_bytes!(8, 16);

#[cfg(test)]
mod tests {
    use crate::{decoder_builder, ThreadsRunner};

    use super::*;

    #[test]
    fn simple() -> Result<(), Box<dyn std::error::Error>> {
        let sample = std::fs::read("samples/sample.jxl")?;
        let parallel_runner = ThreadsRunner::default();
        let decoder = decoder_builder()
            .parallel_runner(&parallel_runner)
            .build()?;

        let img = decoder
            .decode::<u16>(&sample)?
            .into_dynamic_image()
            .ok_or("Failed to create DynamicImage")?;
        let sample_png = image::io::Reader::open("samples/sample.png")?.decode()?;

        assert_eq!(img.to_rgba16(), sample_png.to_rgba16());

        Ok(())
    }

    #[test]
    fn channels() -> Result<(), Box<dyn std::error::Error>> {
        let sample = std::fs::read("samples/sample_grey.jxl")?;
        let parallel_runner = ThreadsRunner::default();
        let mut decoder = decoder_builder()
            .parallel_runner(&parallel_runner)
            .num_channels(1)
            .build()?;

        decoder
            .decode::<u8>(&sample)?
            .into_dynamic_image()
            .ok_or("Failed to create DynamicImage")?;

        decoder.set_num_channels(2);
        decoder
            .decode::<u8>(&sample)?
            .into_dynamic_image()
            .ok_or("Failed to create DynamicImage")?;

        decoder.set_num_channels(3);
        decoder
            .decode::<u8>(&sample)?
            .into_dynamic_image()
            .ok_or("Failed to create DynamicImage")?;

        let mut res = decoder.decode::<u8>(&sample)?;
        res.info.num_channels = 0;
        assert!(res.into_dynamic_image().is_none());

        Ok(())
    }
}
