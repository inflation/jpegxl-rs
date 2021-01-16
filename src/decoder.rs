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
#![allow(non_upper_case_globals)]

//! Decoder of JPEG XL format
//! # Example
//! ```
//! # use jpegxl_rs::*;
//! # || -> Result<(), Box<dyn std::error::Error>> {
//! let mut decoder: JXLDecoder<u8> = decoder_builder().build()?;
//!
//! // Use another pixel data type
//! let mut decoder: JXLDecoder<f32> = decoder_builder().build()?;
//!
//! // Customize pixel format
//! let mut decoder: JXLDecoder<u8> = decoder_builder()
//!                                     .num_channels(3)
//!                                     .endian(Endianness::Big)
//!                                     .align(8)
//!                                     .build()?;
//!
//! // Set custom parallel runner and memory manager
//! use jpegxl_rs::{parallel::ThreadsRunner, memory::MallocManager};
//! let manager = Box::new(MallocManager::default());
//! let runner = Box::new(ThreadsRunner::default());
//! let mut decoder: JXLDecoder<u8> = decoder_builder()
//!                                     .memory_manager(manager)
//!                                     .parallel_runner(runner)
//!                                     .build()?;
//! # Ok(()) };
//! ```

use std::ffi::c_void;
use std::ptr::null;

use jpegxl_sys::*;

pub use super::*;
use crate::{
    error::{check_dec_status, DecodeError},
    memory::*,
    parallel::*,
};

/// Basic Information
pub type BasicInfo = JxlBasicInfo;

struct PrefferedPixelFormat {
    num_channels: Option<u32>,
    endianness: Option<Endianness>,
    align: Option<u64>,
}

/// JPEG XL Decoder
pub struct JXLDecoder<T: PixelType> {
    /// Opaque pointer to the underlying decoder
    dec: *mut JxlDecoder,

    /// Pixel format
    pixel_format: PrefferedPixelFormat,
    _pixel_type: std::marker::PhantomData<T>,

    /// Memory Manager
    _memory_manager: Option<Box<dyn JXLMemoryManager>>,

    /// Parallel Runner
    parallel_runner: Option<Box<dyn JXLParallelRunner>>,
}

impl<T: PixelType> JXLDecoder<T> {
    fn new(
        pixel_format: PrefferedPixelFormat,
        mut memory_manager: Option<Box<dyn JXLMemoryManager>>,
        parallel_runner: Option<Box<dyn JXLParallelRunner>>,
    ) -> Result<Self, DecodeError> {
        let dec = unsafe {
            if let Some(memory_manager) = &mut memory_manager {
                JxlDecoderCreate(&memory_manager.to_manager())
            } else {
                JxlDecoderCreate(null())
            }
        };

        if dec.is_null() {
            return Err(DecodeError::CannotCreateDecoder);
        }

        Ok(Self {
            dec,
            pixel_format,
            _pixel_type: std::marker::PhantomData,
            _memory_manager: memory_manager,
            parallel_runner,
        })
    }

    /// Decode a JPEG XL image.<br />
    /// Currently only support RGB(A)8/16/32 encoded static image. Color info and transformation info are discarded.
    pub fn decode(&mut self, data: &[u8]) -> Result<(BasicInfo, Vec<T>), DecodeError> {
        unsafe {
            if let Some(ref mut runner) = self.parallel_runner {
                check_dec_status(JxlDecoderSetParallelRunner(
                    self.dec,
                    Some(runner.runner()),
                    runner.as_opaque_ptr(),
                ))?
            }

            // Stop after getting the basic info and decoding the image
            check_dec_status(JxlDecoderSubscribeEvents(
                self.dec,
                (JxlDecoderStatus_JXL_DEC_BASIC_INFO | JxlDecoderStatus_JXL_DEC_FULL_IMAGE) as i32,
            ))?;

            let next_in = &mut data.as_ptr();
            let mut avail_in = std::mem::size_of_val(data) as _;

            let mut basic_info = None;
            let mut pixel_format = None;
            let mut buffer = Vec::new();

            let mut status: u32;
            loop {
                status = JxlDecoderProcessInput(self.dec, next_in, &mut avail_in);

                match status {
                    JxlDecoderStatus_JXL_DEC_ERROR => return Err(DecodeError::GenericError),
                    JxlDecoderStatus_JXL_DEC_NEED_MORE_INPUT => {
                        return Err(DecodeError::NeedMoreInput)
                    }

                    // Get the basic info
                    JxlDecoderStatus_JXL_DEC_BASIC_INFO => {
                        let mut info = JxlBasicInfo::new_uninit();
                        check_dec_status(JxlDecoderGetBasicInfo(self.dec, info.as_mut_ptr()))?;
                        let info = info.assume_init();
                        basic_info = Some(info);

                        let mut format = JxlPixelFormat::new_uninit();
                        check_dec_status(JxlDecoderDefaultPixelFormat(
                            self.dec,
                            format.as_mut_ptr(),
                        ))?;
                        let format = format.assume_init();
                        let num_channels = self
                            .pixel_format
                            .num_channels
                            .unwrap_or(format.num_channels);
                        let endianness = self
                            .pixel_format
                            .endianness
                            .map(Endianness::into)
                            .unwrap_or(format.endianness);
                        let align = self.pixel_format.align.unwrap_or(format.align);

                        pixel_format = Some(JxlPixelFormat {
                            num_channels,
                            data_type: T::pixel_type(),
                            endianness,
                            align,
                        })
                    }

                    // Get the output buffer
                    JxlDecoderStatus_JXL_DEC_NEED_IMAGE_OUT_BUFFER => {
                        let mut size: u64 = 0;
                        if let Some(format) = pixel_format {
                            check_dec_status(JxlDecoderImageOutBufferSize(
                                self.dec, &format, &mut size,
                            ))?;

                            buffer = Vec::with_capacity(size as _);
                            buffer.set_len(size as _);
                            check_dec_status(JxlDecoderSetImageOutBuffer(
                                self.dec,
                                &format,
                                buffer.as_mut_ptr() as *mut c_void,
                                size,
                            ))?;
                        } else {
                            return Err(DecodeError::GenericError);
                        };
                    }

                    JxlDecoderStatus_JXL_DEC_FULL_IMAGE => continue,
                    JxlDecoderStatus_JXL_DEC_SUCCESS => {
                        JxlDecoderReset(self.dec);
                        return if let Some(info) = basic_info {
                            Ok((info, buffer))
                        } else {
                            Err(DecodeError::GenericError)
                        };
                    }
                    _ => return Err(DecodeError::UnknownStatus(status)),
                }
            }
        }
    }
}

impl<T: PixelType> Drop for JXLDecoder<T> {
    fn drop(&mut self) {
        unsafe { JxlDecoderDestroy(self.dec) };
    }
}

/// Builder for JXLDecoder
pub struct JXLDecoderBuilder<T: PixelType> {
    pixel_format: PrefferedPixelFormat,
    _pixel_type: std::marker::PhantomData<T>,
    memory_manager: Option<Box<dyn JXLMemoryManager>>,
    parallel_runner: Option<Box<dyn JXLParallelRunner>>,
}

impl<T: PixelType> JXLDecoderBuilder<T> {
    /// Set number of channels for returned result
    pub fn num_channels(mut self, num: u32) -> Self {
        self.pixel_format.num_channels = Some(num);
        self
    }

    /// Set endianness for returned result
    pub fn endian(mut self, endian: Endianness) -> Self {
        self.pixel_format.endianness = Some(endian);
        self
    }

    /// Set align for returned result
    pub fn align(mut self, align: u64) -> Self {
        self.pixel_format.align = Some(align);
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

    /// Consume the builder and get the decoder
    pub fn build(self) -> Result<JXLDecoder<T>, DecodeError> {
        JXLDecoder::new(self.pixel_format, self.memory_manager, self.parallel_runner)
    }
}

/// Return a builder for JXLDecoder
pub fn decoder_builder<T: PixelType>() -> JXLDecoderBuilder<T> {
    JXLDecoderBuilder {
        pixel_format: PrefferedPixelFormat {
            num_channels: None,
            endianness: None,
            align: None,
        },
        _pixel_type: std::marker::PhantomData,
        memory_manager: None,
        parallel_runner: choose_runner(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::image::ImageError;
    #[test]
    fn test_decode() -> Result<(), ImageError> {
        let sample = std::fs::read("test/sample.jxl")?;
        let mut decoder: JXLDecoder<u8> = decoder_builder().build()?;

        let (basic_info, buffer) = decoder.decode(&sample)?;

        assert_eq!(buffer.len(), (basic_info.xsize * basic_info.ysize * 4) as _);

        Ok(())
    }

    #[test]
    fn test_decoder_builder() -> Result<(), ImageError> {
        let sample = std::fs::read("test/sample.jxl")?;
        let mut decoder: JXLDecoder<u8> = decoder_builder()
            .num_channels(3)
            .endian(Endianness::Big)
            .build()?;

        let (basic_info, buffer) = decoder.decode(&sample)?;

        assert_eq!(buffer.len(), (basic_info.xsize * basic_info.ysize * 3) as _);

        Ok(())
    }

    #[test]
    #[cfg(feature = "with-rayon")]
    fn test_rust_runner_decode() -> Result<(), ImageError> {
        let sample = std::fs::read("test/sample.jxl")?;
        let parallel_runner = Box::new(RayonRunner::default());

        let mut decoder: JXLDecoder<u8> =
            decoder_builder().parallel_runner(parallel_runner).build()?;

        let parallel_buffer = decoder.decode(&sample)?;

        decoder = decoder_builder().build()?;
        let single_buffer = decoder.decode(&sample)?;

        assert!(
            parallel_buffer.1 == single_buffer.1,
            "Rust runner should be the same as C++ one"
        );

        Ok(())
    }

    #[test]
    fn test_memory_manager() -> Result<(), Box<dyn std::error::Error>> {
        use crate::memory::*;

        let sample = std::fs::read("test/sample.jxl")?;
        let memory_manager = Box::new(MallocManager::default());

        let mut decoder: JXLDecoder<u8> =
            decoder_builder().memory_manager(memory_manager).build()?;
        let custom_buffer = decoder.decode(&sample)?;

        decoder = decoder_builder().build()?;
        let default_buffer = decoder.decode(&sample)?;

        assert!(
            custom_buffer.1 == default_buffer.1,
            "Custom memory manager should be the same as default one"
        );

        Ok(())
    }
}
