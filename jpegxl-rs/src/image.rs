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

use image::{DynamicImage, ImageBuffer};
use paste::paste;

use crate::decode::{Data, DecoderResult};

/// Extension trait for [`DecoderResult`]
pub trait ToDynamic {
    /// Convert the decoded result to a [`DynamicImage`]
    fn into_dynamic_image(self) -> Option<DynamicImage>;
}

macro_rules! to_dynamic {
    ($info:expr, $data:expr, $x:ident) => {
        ImageBuffer::from_raw($info.width, $info.height, $data).map(DynamicImage::$x)
    };
}

macro_rules! impl_to_dynamic_for_bytes {
    ($info:expr, $data:expr, $b:expr) => {
        paste! {
            match $info.num_channels {
                1 => to_dynamic!($info,$data, [<ImageLuma $b>]),
                2 => to_dynamic!($info,$data, [<ImageLumaA $b>]),
                3 => to_dynamic!($info,$data, [<ImageRgb $b>]),
                4 => to_dynamic!($info,$data, [<ImageRgba $b>]),
                _ => None,
            }
        }
    };
}

impl ToDynamic for DecoderResult {
    fn into_dynamic_image(self) -> Option<DynamicImage> {
        match self.data {
            Data::U8(data) => impl_to_dynamic_for_bytes!(self, data, 8),
            Data::U16(data) => impl_to_dynamic_for_bytes!(self, data, 16),
            Data::F32(data) => match self.num_channels {
                3 => to_dynamic!(self, data, ImageRgb32F),
                4 => to_dynamic!(self, data, ImageRgba32F),
                _ => None,
            },
            Data::F16(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use half::f16;

    use crate::{decoder_builder, ThreadsRunner};

    use super::*;

    #[test]
    fn simple() -> Result<(), Box<dyn std::error::Error>> {
        let sample = std::fs::read("../samples/sample.jxl")?;
        let parallel_runner = ThreadsRunner::default();
        let decoder = decoder_builder()
            .parallel_runner(&parallel_runner)
            .build()?;

        let img = decoder
            .decode_to::<u16>(&sample)?
            .into_dynamic_image()
            .ok_or("Failed to create DynamicImage")?;
        let sample_png = image::io::Reader::open("../samples/sample.png")?.decode()?;

        assert_eq!(img.to_rgba16(), sample_png.to_rgba16());

        Ok(())
    }

    #[test]
    fn channels() {
        let sample = std::fs::read("../samples/sample_grey.jxl").unwrap();
        let parallel_runner = ThreadsRunner::default();
        let mut decoder = decoder_builder()
            .parallel_runner(&parallel_runner)
            .num_channels(1)
            .build()
            .unwrap();

        decoder
            .decode_to::<u8>(&sample)
            .unwrap()
            .into_dynamic_image()
            .unwrap();

        decoder.num_channels = 2;
        decoder
            .decode_to::<u8>(&sample)
            .unwrap()
            .into_dynamic_image()
            .unwrap();

        decoder.num_channels = 3;
        decoder
            .decode_to::<u8>(&sample)
            .unwrap()
            .into_dynamic_image()
            .unwrap();

        let mut res = decoder.decode_to::<u8>(&sample).unwrap();
        res.num_channels = 0;
        assert!(res.into_dynamic_image().is_none());
    }

    #[test]
    fn pixels() {
        let sample = std::fs::read("../samples/sample.jxl").unwrap();
        let parallel_runner = ThreadsRunner::default();
        let decoder = decoder_builder()
            .parallel_runner(&parallel_runner)
            .build()
            .unwrap();

        decoder
            .decode_to::<u8>(&sample)
            .unwrap()
            .into_dynamic_image()
            .unwrap();

        decoder
            .decode_to::<u16>(&sample)
            .unwrap()
            .into_dynamic_image()
            .unwrap();

        assert!(decoder
            .decode_to::<f16>(&sample)
            .unwrap()
            .into_dynamic_image()
            .is_none());

        decoder
            .decode_to::<f32>(&sample)
            .unwrap()
            .into_dynamic_image()
            .unwrap();

        let sample = std::fs::read("../samples/sample_grey.jxl").unwrap();
        assert!(decoder
            .decode_to::<f32>(&sample)
            .unwrap()
            .into_dynamic_image()
            .is_none());
    }
}
