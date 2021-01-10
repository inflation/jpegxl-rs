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
//! # Example
//! ```
//! # use jpegxl_rs::encoder::*;
//! # || -> Result<(), Box<dyn std::error::Error>> {
//! use image::io::Reader as ImageReader;
//! let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgba16();
//! let mut encoder = encoder_builder().build();
//! let buffer = encoder.encode(&sample, sample.width() as u64, sample.height() as u64)?;
//! # Ok(()) };
//! ```
//!
//! Set encoder options
//! ```
//! # || -> Result<(), Box<dyn std::error::Error>> {
//! # use jpegxl_rs::*;
//! let mut encoder = encoder_builder()
//!                     .lossless(true)
//!                     .speed(EncodeSpeed::Falcon)
//!                     .build();
//! // You can change the settings after initialization
//! encoder.set_lossless(false);
//! encoder.set_quality(3.0);
//! # Ok(()) };
//! ```

use std::ffi::c_void;
use std::ptr::null;

use jpegxl_sys::*;

use crate::{
    common::*,
    error::{check_enc_status, EncodeError},
    memory::*,
    parallel::*,
};

/// Encoding speed, default at Squeirrel(7)
#[derive(Debug)]
pub enum EncodeSpeed {
    /// Fastest, 3
    Falcon = 3,
    /// 4
    Cheetah,
    /// 5
    Hare,
    /// 6
    Wombat,
    /// 7
    Squeirrel,
    /// 8
    Kitten,
    /// Slowest, 9
    Tortoise,
}

impl From<EncodeSpeed> for i32 {
    fn from(s: EncodeSpeed) -> Self {
        s as i32
    }
}
struct PrefferedEncoderOptions {
    lossless: Option<bool>,
    speed: Option<EncodeSpeed>,
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
        encoder_options: PrefferedEncoderOptions,
        mut memory_manager: Option<Box<dyn JXLMemoryManager>>,
        parallel_runner: Option<Box<dyn JXLParallelRunner>>,
    ) -> Self {
        let enc = unsafe {
            if let Some(memory_manager) = &mut memory_manager {
                JxlEncoderCreate(&memory_manager.to_manager())
            } else {
                JxlEncoderCreate(null())
            }
        };

        let options_ptr = unsafe { JxlEncoderOptionsCreate(enc, null()) };

        let encoder = Self {
            enc,
            pixel_format,
            options_ptr,
            _memory_manager: memory_manager,
            parallel_runner,
        };

        let PrefferedEncoderOptions {
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
        encoder
    }

    /// Set lossless mode. Default is lossy.
    pub fn set_lossless(&self, lossless: bool) {
        unsafe { JxlEncoderOptionsSetLossless(self.options_ptr, lossless.into()) };
    }

    /// Set speed.
    /// Default: EncodeSpeed::Squirrel (7).
    pub fn set_speed(&self, speed: EncodeSpeed) {
        unsafe { JxlEncoderOptionsSetEffort(self.options_ptr, speed.into()) };
    }

    /// Set quality for lossy compression: target max butteraugli distance, lower = higher quality.
    ///  Range: 0 .. 15.<br />
    ///    0.0 = mathematically lossless (however, use `set_lossless` to use true lossless). <br />
    ///    1.0 = visually lossless. <br />
    ///    Recommended range: 0.5 .. 3.0. <br />
    ///    Default value: 1.0. <br />
    ///    If `set_lossless` is used, this value is unused and implied to be 0.
    pub fn set_quality(&self, quality: f32) {
        unsafe { JxlEncoderOptionsSetDistance(self.options_ptr, quality) };
    }

    /// Encode a JPEG XL image.<br />
    pub fn encode<T: PixelType>(
        &mut self,
        data: &[T],
        xsize: u64,
        ysize: u64,
    ) -> Result<Vec<u8>, EncodeError> {
        self.pixel_format.data_type = T::pixel_type();

        unsafe {
            if let Some(ref mut runner) = self.parallel_runner {
                check_enc_status(JxlEncoderSetParallelRunner(
                    self.enc,
                    Some(runner.runner()),
                    runner.as_opaque_ptr(),
                ))?
            }

            check_enc_status(JxlEncoderSetDimensions(self.enc, xsize, ysize))?;

            check_enc_status(JxlEncoderAddImageFrame(
                self.options_ptr,
                &self.pixel_format,
                data.as_ptr() as *mut c_void,
                std::mem::size_of_val(data) as u64,
            ))?;

            const CHUNK_SIZE: usize = 1024 * 512; // 512 KB is a good initial value
            let mut buffer = vec![0u8; CHUNK_SIZE];
            let mut next_out = buffer.as_mut_ptr();
            let mut avail_out = CHUNK_SIZE;

            let mut status;
            loop {
                status = JxlEncoderProcessOutput(
                    self.enc,
                    &mut next_out as *mut *mut u8,
                    &mut (avail_out as u64) as *mut u64,
                );

                if status != JxlEncoderStatus_JXL_ENC_NEED_MORE_OUTPUT {
                    break;
                }

                let offset = next_out as usize - buffer.as_ptr() as usize;
                buffer.resize(buffer.len() * 2, 0);
                next_out = buffer.as_mut_ptr().add(offset);
                avail_out = buffer.len() - offset;
            }
            buffer.truncate(next_out as usize - buffer.as_ptr() as usize);
            check_enc_status(status)?;

            Ok(buffer)
        }
    }
}

impl Drop for JXLEncoder {
    fn drop(&mut self) {
        unsafe { JxlEncoderDestroy(self.enc) };
    }
}

/// Builder for JXLDecoder
pub struct JXLEncoderBuilder {
    pixel_format: JxlPixelFormat,
    options: PrefferedEncoderOptions,
    memory_manager: Option<Box<dyn JXLMemoryManager>>,
    parallel_runner: Option<Box<dyn JXLParallelRunner>>,
}

impl JXLEncoderBuilder {
    /// Set number of channels
    pub fn num_channels(mut self, num: u32) -> Self {
        self.pixel_format.num_channels = num;
        self
    }

    /// Set endianness
    pub fn endian(mut self, endian: Endianness) -> Self {
        self.pixel_format.endianness = endian.into();
        self
    }

    /// Set align
    pub fn align(mut self, align: u64) -> Self {
        self.pixel_format.align = align;
        self
    }

    /// Set lossless
    pub fn lossless(mut self, flag: bool) -> Self {
        self.options.lossless = Some(flag);
        self
    }

    /// Set speed
    pub fn speed(mut self, speed: EncodeSpeed) -> Self {
        self.options.speed = Some(speed);
        self
    }

    /// Set quality
    pub fn quality(mut self, quality: f32) -> Self {
        self.options.quality = Some(quality);
        self
    }

    /// Set memory manager
    pub fn memory_manager(mut self, memory_manager: Box<dyn JXLMemoryManager>) -> Self {
        self.memory_manager = Some(memory_manager);
        self
    }

    /// Set parallel runner
    pub fn parallel_runner(mut self, parallel_runner: Box<dyn JXLParallelRunner>) -> Self {
        self.parallel_runner = Some(parallel_runner);
        self
    }

    /// Consume the builder and get the encoder
    pub fn build(self) -> JXLEncoder {
        JXLEncoder::new(
            self.pixel_format,
            self.options,
            self.memory_manager,
            self.parallel_runner,
        )
    }
}

/// Return a builder for JXLEncoder
pub fn encoder_builder() -> JXLEncoderBuilder {
    JXLEncoderBuilder {
        pixel_format: JxlPixelFormat {
            num_channels: 4,
            data_type: u8::pixel_type(),
            endianness: Endianness::Native.into(),
            align: 0,
        },
        options: PrefferedEncoderOptions {
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
            .speed(EncodeSpeed::Falcon)
            .num_channels(3)
            .build();

        encoder.encode(
            sample.as_raw(),
            sample.width() as u64,
            sample.height() as u64,
        )?;

        Ok(())
    }

    #[test]
    fn test_encode_builder() -> Result<(), image::ImageError> {
        let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgb8();
        let mut encoder = encoder_builder()
            .num_channels(3)
            .lossless(false)
            .speed(EncodeSpeed::Falcon)
            .quality(3.0)
            .build();

        encoder.encode(
            sample.as_raw(),
            sample.width() as u64,
            sample.height() as u64,
        )?;

        Ok(())
    }

    #[test]
    #[cfg(feature = "with-rayon")]
    fn test_rust_runner_encode() -> Result<(), Box<dyn std::error::Error>> {
        let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgba16();
        let parallel_runner = Box::new(RayonRunner::default());

        let mut encoder = encoder_builder()
            .speed(EncodeSpeed::Falcon)
            .parallel_runner(parallel_runner)
            .build();

        let parallel_buffer = encoder.encode(
            sample.as_raw(),
            sample.width() as u64,
            sample.height() as u64,
        )?;

        encoder = encoder_builder().speed(EncodeSpeed::Falcon).build();
        let single_buffer = encoder.encode(
            sample.as_raw(),
            sample.width() as u64,
            sample.height() as u64,
        )?;

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
            .speed(EncodeSpeed::Falcon)
            .memory_manager(memory_manager)
            .build();
        let custom_buffer = encoder.encode(
            sample.as_raw(),
            sample.width() as u64,
            sample.height() as u64,
        )?;

        encoder = encoder_builder().speed(EncodeSpeed::Falcon).build();
        let default_buffer = encoder.encode(
            sample.as_raw(),
            sample.width() as u64,
            sample.height() as u64,
        )?;

        assert!(
            custom_buffer == default_buffer,
            "Custom memory manager should be the same as default one"
        );

        Ok(())
    }
}
