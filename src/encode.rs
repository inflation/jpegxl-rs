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

//! Encoder of JPEG XL format

#[allow(clippy::wildcard_imports)]
use jpegxl_sys::*;
use std::{convert::TryInto as _, ptr::null};

use crate::{
    common::{Endianness, PixelType},
    errors::{check_enc_status, EncodeError},
    memory::JxlMemoryManager,
    parallel::{choose_runner, JxlParallelRunner},
};

/// Encoding speed, default at Squirrel(7)
#[derive(Debug)]
pub enum EncoderSpeed {
    /// Fastest, 3
    Falcon = 3,
    /// 4
    Cheetah,
    /// 5
    Hare,
    /// 6
    Wombat,
    /// 7
    Squirrel,
    /// 8
    Kitten,
    /// Slowest, 9
    Tortoise,
}

/// Encoding color profile
#[derive(Debug)]
pub enum ColorEncoding {
    /// SRGB, default for uint pixel types
    SRGB,
    /// Linear SRGB, default for float pixel types
    LinearSRGB,
}

/// JPEG XL Encoder
pub struct JxlEncoder {
    /// Opaque pointer to the underlying encoder
    enc: *mut jpegxl_sys::JxlEncoder,
    /// Opaque pointer to the encoder options
    options_ptr: *mut JxlEncoderOptions,

    /// Pixel format
    pub pixel_format: JxlPixelFormat,

    /// Color Encoding
    color_encoding: Option<JxlColorEncoding>,

    /// Memory Manager
    #[allow(dead_code)]
    memory_manager: Option<Box<dyn JxlMemoryManager>>,

    /// Parallel Runner
    pub parallel_runner: Option<Box<dyn JxlParallelRunner>>,
}

impl JxlEncoder {
    fn new(
        pixel_format: JxlPixelFormat,
        encoder_options: PreferredEncoderOptions,
        mut memory_manager: Option<Box<dyn JxlMemoryManager>>,
        parallel_runner: Option<Box<dyn JxlParallelRunner>>,
    ) -> Result<Self, EncodeError> {
        let enc = unsafe {
            memory_manager.as_mut().map_or_else(
                || JxlEncoderCreate(null()),
                |memory_manager| JxlEncoderCreate(&memory_manager.as_manager()),
            )
        };

        if enc.is_null() {
            return Err(EncodeError::CannotCreateEncoder);
        }

        let options_ptr = unsafe { JxlEncoderOptionsCreate(enc, null()) };

        let PreferredEncoderOptions {
            lossless,
            speed,
            quality,
            color_encoding,
        } = encoder_options;

        let color_encoding = color_encoding.map(|c| unsafe {
            let mut color_encoding = JxlColorEncoding::new_uninit();
            match c {
                ColorEncoding::SRGB => {
                    JxlColorEncodingSetToSRGB(color_encoding.as_mut_ptr(), false.into())
                }
                ColorEncoding::LinearSRGB => {
                    JxlColorEncodingSetToLinearSRGB(color_encoding.as_mut_ptr(), false.into())
                }
            }
            color_encoding.assume_init()
        });

        let mut encoder = Self {
            enc,
            pixel_format,
            options_ptr,
            color_encoding,
            memory_manager,
            parallel_runner,
        };

        if let Some(l) = lossless {
            encoder.set_lossless(l);
        }
        if let Some(s) = speed {
            encoder.set_speed(s);
        }
        if let Some(q) = quality {
            encoder.set_quality(q);
        }
        Ok(encoder)
    }

    /// Set lossless mode. Default is lossy.
    pub fn set_lossless(&mut self, lossless: bool) {
        unsafe { JxlEncoderOptionsSetLossless(self.options_ptr, lossless.into()) };
    }

    /// Set speed.
    /// Default: `[EncodeSpeed::Squirrel] (7)`.
    pub fn set_speed(&mut self, speed: EncoderSpeed) {
        unsafe { JxlEncoderOptionsSetEffort(self.options_ptr, speed as i32) };
    }

    /// Set quality for lossy compression: target max butteraugli distance, lower = higher quality.
    ///  Range: 0 .. 15.<br />
    ///    0.0 = mathematically lossless (however, use `set_lossless` to use true lossless). <br />
    ///    1.0 = visually lossless. <br />
    ///    Recommended range: 0.5 .. 3.0. <br />
    ///    Default value: 1.0. <br />
    ///    If `set_lossless` is used, this value is unused and implied to be 0.
    pub fn set_quality(&mut self, quality: f32) {
        unsafe { JxlEncoderOptionsSetDistance(self.options_ptr, quality) };
    }

    // Internal encoder setup
    fn _encode<T: PixelType, U: PixelType>(
        &mut self,
        data: &[T],
        x_size: u32,
        y_size: u32,
        is_jpeg: bool,
    ) -> Result<Vec<U>, EncodeError> {
        unsafe {
            if let Some(ref mut runner) = self.parallel_runner {
                check_enc_status(JxlEncoderSetParallelRunner(
                    self.enc,
                    Some(runner.runner()),
                    runner.as_opaque_ptr(),
                ))?
            }

            if let Some(c) = self.color_encoding {
                JxlEncoderSetColorEncoding(self.enc, &c);
            }

            let mut basic_info = JxlBasicInfo::new_uninit().assume_init();
            basic_info.xsize = x_size;
            basic_info.ysize = y_size;

            let (bits, exp) = U::bits_per_sample();

            basic_info.bits_per_sample = bits;
            basic_info.exponent_bits_per_sample = exp;

            // TODO: Handle alpha channel better when upstream improves
            //       Currently upstream only supports channels of 2 or 4 for alpha
            if self.pixel_format.num_channels % 2 == 0 {
                basic_info.alpha_bits = bits;
                basic_info.alpha_exponent_bits = exp;
            } else {
                basic_info.alpha_bits = 0;
                basic_info.alpha_exponent_bits = 0;
            }
            check_enc_status(JxlEncoderSetBasicInfo(self.enc, &basic_info))?;

            check_enc_status(if is_jpeg {
                JxlEncoderAddJPEGFrame(
                    self.options_ptr,
                    data.as_ptr() as _,
                    std::mem::size_of_val(data) as _,
                )
            } else {
                JxlEncoderAddImageFrame(
                    self.options_ptr,
                    &self.pixel_format,
                    data.as_ptr() as _,
                    std::mem::size_of_val(data) as _,
                )
            })?;

            let chunk_size = 1024 * 512; // 512 KB is a good initial value
            let mut buffer: Vec<U> = {
                let mut buffer = Vec::with_capacity(chunk_size);
                buffer.set_len(chunk_size);
                buffer
            };
            let mut next_out = buffer.as_mut_ptr() as *mut u8;
            let mut avail_out = chunk_size;

            let mut status;
            loop {
                status = JxlEncoderProcessOutput(self.enc, &mut next_out, &mut (avail_out as u64));

                if status != JxlEncoderStatus_JXL_ENC_NEED_MORE_OUTPUT {
                    break;
                }

                let offset = next_out
                    .offset_from(buffer.as_ptr() as *const u8)
                    .try_into()
                    .unwrap();
                buffer.resize(buffer.len() * 2, U::default());
                next_out = (buffer.as_mut_ptr() as *mut u8).add(offset);
                avail_out = buffer.len() - offset;
            }
            buffer.truncate(
                next_out
                    .offset_from(buffer.as_ptr() as *const u8)
                    .try_into()
                    .unwrap(),
            );
            check_enc_status(status)?;

            Ok(buffer)
        }
    }

    /// Encode a JPEG XL image from existing raw JPEG data. Only support pixel type  `u8`
    /// # Errors
    /// Return [`EncodeError`] if internal encoder fails to encode
    pub fn encode_jpeg(
        &mut self,
        data: &[u8],
        x_size: u32,
        y_size: u32,
    ) -> Result<Vec<u8>, EncodeError> {
        self._encode::<u8, u8>(data, x_size, y_size, true)
    }

    /// Encode a JPEG XL image from pixels
    /// # Errors
    /// Return [`EncodeError`] if internal encoder fails to encode
    pub fn encode<T: PixelType, U: PixelType>(
        &mut self,
        data: &[T],
        x_size: u32,
        y_size: u32,
    ) -> Result<Vec<U>, EncodeError> {
        self.pixel_format.data_type = T::pixel_type();

        self._encode(data, x_size, y_size, false)
    }
}

impl Drop for JxlEncoder {
    fn drop(&mut self) {
        unsafe { JxlEncoderDestroy(self.enc) };
    }
}

struct PreferredEncoderOptions {
    lossless: Option<bool>,
    speed: Option<EncoderSpeed>,
    quality: Option<f32>,
    color_encoding: Option<ColorEncoding>,
}

/// Builder for [`JxlEncoder`]
pub struct JxlEncoderBuilder {
    pixel_format: JxlPixelFormat,
    options: PreferredEncoderOptions,
    memory_manager: Option<Box<dyn JxlMemoryManager>>,
    parallel_runner: Option<Box<dyn JxlParallelRunner>>,
}

impl JxlEncoderBuilder {
    /// Set number of channels
    #[must_use]
    pub fn num_channels(mut self, num: u32) -> Self {
        self.pixel_format.num_channels = num;
        self
    }

    /// Set endianness
    #[must_use]
    pub fn endian(mut self, endian: Endianness) -> Self {
        self.pixel_format.endianness = endian.into();
        self
    }

    /// Set align
    #[must_use]
    pub fn align(mut self, align: u64) -> Self {
        self.pixel_format.align = align;
        self
    }

    /// Set lossless
    #[must_use]
    pub fn lossless(mut self, flag: bool) -> Self {
        self.options.lossless = Some(flag);
        self
    }

    /// Set speed
    #[must_use]
    pub fn speed(mut self, speed: EncoderSpeed) -> Self {
        self.options.speed = Some(speed);
        self
    }

    /// Set quality
    #[must_use]
    pub fn quality(mut self, quality: f32) -> Self {
        self.options.quality = Some(quality);
        self
    }

    /// Set color encoding
    #[must_use]
    pub fn color_encoding(mut self, encoding: ColorEncoding) -> Self {
        self.options.color_encoding = Some(encoding);
        self
    }

    /// Set memory manager
    #[must_use]
    pub fn memory_manager(mut self, memory_manager: Box<dyn JxlMemoryManager>) -> Self {
        self.memory_manager = Some(memory_manager);
        self
    }

    /// Set parallel runner
    #[must_use]
    pub fn parallel_runner(mut self, parallel_runner: Box<dyn JxlParallelRunner>) -> Self {
        self.parallel_runner = Some(parallel_runner);
        self
    }

    /// Consume the builder and get the encoder
    /// # Errors
    /// Return [`EncodeError::CannotCreateEncoder`] if it fails to create the encoder
    pub fn build(self) -> Result<JxlEncoder, EncodeError> {
        JxlEncoder::new(
            self.pixel_format,
            self.options,
            self.memory_manager,
            self.parallel_runner,
        )
    }
}

/// Return a [`JxlEncoderBuilder`] with default settings
#[must_use]
pub fn encoder_builder() -> JxlEncoderBuilder {
    JxlEncoderBuilder {
        pixel_format: JxlPixelFormat {
            num_channels: 4,
            data_type: u8::pixel_type(),
            endianness: Endianness::Native.into(),
            align: 0,
        },
        options: PreferredEncoderOptions {
            lossless: None,
            speed: None,
            quality: None,
            color_encoding: None,
        },
        memory_manager: None,
        parallel_runner: choose_runner(),
    }
}

#[cfg(test)]
mod tests {
    use crate::decoder_builder;

    use super::*;

    use image::io::Reader as ImageReader;
    #[test]
    fn test_encode() -> Result<(), image::ImageError> {
        let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgb8();
        let mut encoder = encoder_builder()
            .speed(EncoderSpeed::Falcon)
            .num_channels(3)
            .build()?;

        let result: Vec<f32> = encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

        let mut decoder = decoder_builder::<f32>().build()?;
        decoder.decode(unsafe {
            std::slice::from_raw_parts(
                result.as_ptr() as *const u8,
                result.len() * std::mem::size_of::<f32>(),
            )
        })?;

        Ok(())
    }

    #[test]
    fn test_encode_jpeg() -> Result<(), image::ImageError> {
        let sample = std::fs::read("test/sample.jpg")?;
        let sample_image = ImageReader::new(std::io::Cursor::new(sample.clone()))
            .with_guessed_format()?
            .decode()?
            .to_rgb8();
        let mut encoder = encoder_builder()
            .speed(EncoderSpeed::Falcon)
            .num_channels(3)
            .build()?;

        encoder.encode_jpeg(&sample, sample_image.width(), sample_image.height())?;

        Ok(())
    }

    #[test]
    fn test_encoder_builder() -> Result<(), image::ImageError> {
        let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgb8();
        let mut encoder = encoder_builder()
            .num_channels(3)
            .lossless(false)
            .speed(EncoderSpeed::Falcon)
            .quality(3.0)
            .color_encoding(ColorEncoding::LinearSRGB)
            .build()?;

        encoder.encode::<_, u8>(sample.as_raw(), sample.width(), sample.height())?;

        Ok(())
    }

    #[test]
    fn test_memory_manager() -> Result<(), Box<dyn std::error::Error>> {
        use crate::memory::*;

        let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgba16();
        let memory_manager = Box::new(MallocManager::default());

        let mut encoder = encoder_builder()
            .speed(EncoderSpeed::Falcon)
            .memory_manager(memory_manager)
            .build()?;
        let custom_buffer: Vec<u8> =
            encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

        let mut encoder = encoder_builder().speed(EncoderSpeed::Falcon).build()?;
        let default_buffer: Vec<u8> =
            encoder.encode(sample.as_raw(), sample.width(), sample.height())?;

        assert!(
            custom_buffer == default_buffer,
            "Custom memory manager should be the same as default one"
        );

        Ok(())
    }
}
