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
//! # Ok(())
//! # };
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
    /// Create a encoder.
    pub fn new(
        pixel_format: JxlPixelFormat,
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

        Self {
            enc,
            pixel_format,
            options_ptr,
            _memory_manager: memory_manager,
            parallel_runner,
        }
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
        JXLEncoder::new(self.pixel_format, self.memory_manager, self.parallel_runner)
    }
}

/// Return a builder for JXLEncoder
pub fn encoder_builder() -> JXLEncoderBuilder {
    let runner: Box<dyn JXLParallelRunner> = if cfg!(feature = "without-threads") {
        Box::new(ParallelRunner::default())
    } else {
        Box::new(ThreadsRunner::default())
    };

    JXLEncoderBuilder {
        pixel_format: JxlPixelFormat {
            num_channels: 4,
            data_type: u8::pixel_type(),
            endianness: Endianness::Native.into(),
            align: 0,
        },
        memory_manager: None,
        parallel_runner: Some(runner),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use image::io::Reader as ImageReader;
    #[test]
    fn test_encode() -> Result<(), image::ImageError> {
        let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgb8();
        let mut encoder = encoder_builder().num_channels(3).build();
        encoder.set_speed(EncodeSpeed::Falcon);

        let _buffer = encoder.encode(
            sample.as_raw(),
            sample.width() as u64,
            sample.height() as u64,
        )?;

        Ok(())
    }

    #[test]
    fn test_rust_runner_encode() -> Result<(), Box<dyn std::error::Error>> {
        let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgba16();
        let parallel_runner = Box::new(ParallelRunner::default());

        let mut encoder = encoder_builder().parallel_runner(parallel_runner).build();
        encoder.set_speed(EncodeSpeed::Falcon);

        let parallel_buffer = encoder.encode(
            sample.as_raw(),
            sample.width() as u64,
            sample.height() as u64,
        )?;

        encoder = encoder_builder().build();
        encoder.set_speed(EncodeSpeed::Falcon);
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
        use crate::memory::JXLMemoryManager;
        use std::alloc::{GlobalAlloc, Layout, System};

        #[derive(Debug)]
        struct MallocManager {
            layout: Layout,
        }

        impl JXLMemoryManager for MallocManager {
            fn alloc(&self) -> Option<AllocFn> {
                unsafe extern "C" fn alloc(opaque: *mut c_void, size: size_t) -> *mut c_void {
                    println!("Custom alloc");
                    let layout = Layout::from_size_align(size as usize, 8).unwrap();
                    let address = System.alloc(layout);

                    let manager = (opaque as *mut MallocManager).as_mut().unwrap();
                    manager.layout = layout;

                    address as *mut c_void
                }

                Some(alloc)
            }

            fn free(&self) -> Option<FreeFn> {
                unsafe extern "C" fn free(opaque: *mut c_void, address: *mut c_void) {
                    println!("Custom dealloc");
                    let layout = (opaque as *mut MallocManager).as_mut().unwrap().layout;
                    System.dealloc(address as *mut u8, layout);
                }

                Some(free)
            }
        }

        let sample = ImageReader::open("test/sample.png")?.decode()?.to_rgba16();
        let memory_manager = Box::new(MallocManager {
            layout: Layout::from_size_align(0, 8)?,
        });

        let mut encoder = encoder_builder().memory_manager(memory_manager).build();
        encoder.set_speed(EncodeSpeed::Falcon);
        let custom_buffer = encoder.encode(
            sample.as_raw(),
            sample.width() as u64,
            sample.height() as u64,
        )?;

        encoder = encoder_builder().build();
        encoder.set_speed(EncodeSpeed::Falcon);
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
