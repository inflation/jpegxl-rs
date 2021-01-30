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
    memory::JXLMemoryManager,
    parallel::{choose_runner, JXLParallelRunner},
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

struct PreferredEncoderOptions {
    lossless: Option<bool>,
    speed: Option<EncoderSpeed>,
    quality: Option<f32>,
}

/// JPEG XL Encoder
pub struct JXLEncoder {
    /// Opaque pointer to the underlying encoder
    enc: *mut JxlEncoder,
    /// Opaque pointer to the encoder options
    options_ptr: *mut JxlEncoderOptions,

    /// Pixel format
    pub pixel_format: JxlPixelFormat,

    /// Memory Manager
    _memory_manager: Option<Box<dyn JXLMemoryManager>>,

    /// Parallel Runner
    pub parallel_runner: Option<Box<dyn JXLParallelRunner>>,
}

impl JXLEncoder {
    fn new(
        pixel_format: JxlPixelFormat,
        encoder_options: PreferredEncoderOptions,
        mut memory_manager: Option<Box<dyn JXLMemoryManager>>,
        parallel_runner: Option<Box<dyn JXLParallelRunner>>,
    ) -> Result<Self, EncodeError> {
        let enc = unsafe {
            memory_manager.as_mut().map_or_else(
                || JxlEncoderCreate(null()),
                |memory_manager| JxlEncoderCreate(&memory_manager.to_manager()),
            )
        };

        if enc.is_null() {
            return Err(EncodeError::CannotCreateEncoder);
        }

        let options_ptr = unsafe { JxlEncoderOptionsCreate(enc, null()) };

        let mut encoder = Self {
            enc,
            pixel_format,
            options_ptr,
            _memory_manager: memory_manager,
            parallel_runner,
        };

        let PreferredEncoderOptions {
            lossless,
            speed,
            quality,
        } = encoder_options;

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
    fn _encode<T: PixelType>(
        &mut self,
        data: &[T],
        x_size: u64,
        y_size: u64,
        is_jpeg: bool,
    ) -> Result<Vec<u8>, EncodeError> {
        unsafe {
            if let Some(ref mut runner) = self.parallel_runner {
                check_enc_status(JxlEncoderSetParallelRunner(
                    self.enc,
                    Some(runner.runner()),
                    runner.as_opaque_ptr(),
                ))?
            }

            check_enc_status(JxlEncoderSetDimensions(self.enc, x_size, y_size))?;

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
            let mut buffer = {
                let mut buffer = Vec::with_capacity(chunk_size);
                buffer.set_len(chunk_size);
                buffer
            };
            let mut next_out: *mut u8 = buffer.as_mut_ptr();
            let mut avail_out = chunk_size;

            let mut status;
            loop {
                status = JxlEncoderProcessOutput(
                    self.enc,
                    &mut next_out as _,
                    &mut (avail_out as u64) as _,
                );

                if status != JxlEncoderStatus_JXL_ENC_NEED_MORE_OUTPUT {
                    break;
                }

                let offset = next_out.offset_from(buffer.as_ptr()).try_into().unwrap();
                buffer.resize(buffer.len() * 2, 0);
                next_out = buffer.as_mut_ptr().add(offset);
                avail_out = buffer.len() - offset;
            }
            buffer.truncate(next_out.offset_from(buffer.as_ptr()).try_into().unwrap());
            check_enc_status(status)?;

            Ok(buffer)
        }
    }

    /// Encode a JPEG XL image from existing raw JPEG data
    /// # Errors
    /// Return `[EncodeError]` if internal encoder fails to encode
    pub fn encode_jpeg(
        &mut self,
        data: &[u8],
        x_size: u64,
        y_size: u64,
    ) -> Result<Vec<u8>, EncodeError> {
        self._encode(data, x_size, y_size, true)
    }

    /// Encode a JPEG XL image from pixels
    /// # Errors
    /// Return `[EncodeError]` if internal encoder fails to encode
    pub fn encode<T: PixelType>(
        &mut self,
        data: &[T],
        x_size: u64,
        y_size: u64,
    ) -> Result<Vec<u8>, EncodeError> {
        self.pixel_format.data_type = T::pixel_type();

        self._encode(data, x_size, y_size, false)
    }
}

impl Drop for JXLEncoder {
    fn drop(&mut self) {
        unsafe { JxlEncoderDestroy(self.enc) };
    }
}

/// Builder for [`JXLEncoder`]
pub struct JXLEncoderBuilder {
    pixel_format: JxlPixelFormat,
    options: PreferredEncoderOptions,
    memory_manager: Option<Box<dyn JXLMemoryManager>>,
    parallel_runner: Option<Box<dyn JXLParallelRunner>>,
}

impl JXLEncoderBuilder {
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

    /// Set memory manager
    #[must_use]
    pub fn memory_manager(mut self, memory_manager: Box<dyn JXLMemoryManager>) -> Self {
        self.memory_manager = Some(memory_manager);
        self
    }

    /// Set parallel runner
    #[must_use]
    pub fn parallel_runner(mut self, parallel_runner: Box<dyn JXLParallelRunner>) -> Self {
        self.parallel_runner = Some(parallel_runner);
        self
    }

    /// Consume the builder and get the encoder
    /// # Errors
    /// Return `[EncodeError::CannotCreateEncoder]` if it fails to create the encoder
    pub fn build(self) -> Result<JXLEncoder, EncodeError> {
        JXLEncoder::new(
            self.pixel_format,
            self.options,
            self.memory_manager,
            self.parallel_runner,
        )
    }
}

/// Return a `[JXLEncoderBuilder]` with default settings
#[must_use]
pub fn encoder_builder() -> JXLEncoderBuilder {
    JXLEncoderBuilder {
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
        },
        memory_manager: None,
        parallel_runner: choose_runner(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use image::io::Reader as ImageReader;
    #[test]
    fn test_encode() -> Result<(), image::ImageError> {
        let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgb8();
        let mut encoder = encoder_builder()
            .speed(EncoderSpeed::Falcon)
            .num_channels(3)
            .build()?;

        encoder.encode(sample.as_raw(), sample.width() as _, sample.height() as _)?;

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

        encoder.encode_jpeg(
            &sample,
            sample_image.width() as _,
            sample_image.height() as _,
        )?;

        Ok(())
    }

    #[test]
    fn test_encode_builder() -> Result<(), image::ImageError> {
        let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgb8();
        let mut encoder = encoder_builder()
            .num_channels(3)
            .lossless(false)
            .speed(EncoderSpeed::Falcon)
            .quality(3.0)
            .build()?;

        encoder.encode(sample.as_raw(), sample.width() as _, sample.height() as _)?;

        Ok(())
    }

    #[test]
    #[cfg(feature = "with-rayon")]
    fn test_rust_runner_encode() -> Result<(), Box<dyn std::error::Error>> {
        use crate::parallel::RayonRunner;

        let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgba16();
        let parallel_runner = Box::new(RayonRunner::default());

        let mut encoder = encoder_builder()
            .speed(EncoderSpeed::Falcon)
            .parallel_runner(parallel_runner)
            .build()?;

        let parallel_buffer =
            encoder.encode(sample.as_raw(), sample.width() as _, sample.height() as _)?;

        encoder = encoder_builder().speed(EncoderSpeed::Falcon).build()?;
        let single_buffer =
            encoder.encode(sample.as_raw(), sample.width() as _, sample.height() as _)?;

        assert!(
            parallel_buffer == single_buffer,
            "Rust runner should be the same as C++ one"
        );

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
        let custom_buffer =
            encoder.encode(sample.as_raw(), sample.width() as _, sample.height() as _)?;

        encoder = encoder_builder().speed(EncoderSpeed::Falcon).build()?;
        let default_buffer =
            encoder.encode(sample.as_raw(), sample.width() as _, sample.height() as _)?;

        assert!(
            custom_buffer == default_buffer,
            "Custom memory manager should be the same as default one"
        );

        Ok(())
    }
}
