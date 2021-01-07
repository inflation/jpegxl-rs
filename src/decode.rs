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

use std::ffi::c_void;
use std::ptr::null;

use jpegxl_sys::*;

use crate::{
    common::*,
    error::{check_dec_status, JXLDecodeError},
    memory::*,
    parallel::*,
};

/// JPEG XL Decoder
pub struct JXLDecoder<T: PixelType> {
    /// Opaque pointer to the underlying decoder
    dec: *mut JxlDecoder,

    /// Pixel format
    pub pixel_format: JxlPixelFormat,
    _pixel_type: std::marker::PhantomData<T>,

    /// Memory Manager
    pub memory_manager: Option<Box<dyn JXLMemoryManager>>,

    /// Parallel Runner
    pub parallel_runner: Option<Box<dyn JXLParallelRunner>>,
}

impl<T: PixelType> JXLDecoder<T> {
    /// Create a decoder.<br/>
    /// Memory manager and Parallel runner API are WIP.
    pub fn new(
        num_channels: u32,
        endianness: JXLEndianness,
        align: u64,
        mut memory_manager: Option<Box<dyn JXLMemoryManager>>,
        parallel_runner: Option<Box<dyn JXLParallelRunner>>,
    ) -> Self {
        let pixel_format = JxlPixelFormat {
            num_channels,
            data_type: T::pixel_type(),
            endianness: endianness.into(),
            align,
        };

        let dec = unsafe {
            if let Some(memory_manager) = &mut memory_manager {
                JxlDecoderCreate(&memory_manager.to_manager())
            } else {
                JxlDecoderCreate(null())
            }
        };

        Self {
            dec,
            pixel_format,
            _pixel_type: std::marker::PhantomData,
            memory_manager,
            parallel_runner,
        }
    }

    /// Decode a JPEG XL image.<br />
    /// Currently only support RGB(A)8/16/32 encoded static image. Color info and transformation info are discarded.
    /// # Example
    /// ```
    /// # use jpegxl_rs::*;
    /// # || -> Result<(), Box<dyn std::error::Error>> {
    /// let sample = std::fs::read("test/sample.jxl")?;
    /// let mut decoder: JXLDecoder<u8> = JXLDecoder::default();
    /// let (info, buffer) = decoder.decode(&sample)?;
    /// # Ok(())
    /// # };
    /// ```
    pub fn decode(&mut self, data: &[u8]) -> Result<(JXLBasicInfo, Vec<T>), JXLDecodeError> {
        // Stop after getting the basic info and decoding the image
        unsafe {
            check_dec_status(JxlDecoderSubscribeEvents(
                self.dec,
                (JxlDecoderStatus_JXL_DEC_BASIC_INFO | JxlDecoderStatus_JXL_DEC_FULL_IMAGE) as i32,
            ))?;
        }

        let next_in = &mut data.as_ptr();
        let mut avail_in = data.len() as u64;

        let mut basic_info: Option<JXLBasicInfo> = None;
        let mut buffer: Vec<T> = Vec::new();

        unsafe {
            let mut status: u32;
            loop {
                status = JxlDecoderProcessInput(self.dec, next_in, &mut avail_in);

                match status {
                    JxlDecoderStatus_JXL_DEC_ERROR => return Err(JXLDecodeError::GenericError),
                    JxlDecoderStatus_JXL_DEC_NEED_MORE_INPUT => {
                        return Err(JXLDecodeError::NeedMoreInput)
                    }

                    // Get the basic info
                    JxlDecoderStatus_JXL_DEC_BASIC_INFO => {
                        let mut info = JxlBasicInfo::new_uninit();
                        check_dec_status(JxlDecoderGetBasicInfo(self.dec, info.as_mut_ptr()))?;
                        basic_info = Some(info.assume_init());
                    }

                    // Get the output buffer
                    JxlDecoderStatus_JXL_DEC_NEED_IMAGE_OUT_BUFFER => {
                        let mut size: u64 = 0;
                        check_dec_status(JxlDecoderImageOutBufferSize(
                            self.dec,
                            &self.pixel_format,
                            &mut size,
                        ))?;

                        buffer.resize(size as usize, T::default());
                        check_dec_status(JxlDecoderSetImageOutBuffer(
                            self.dec,
                            &self.pixel_format,
                            buffer.as_mut_ptr() as *mut c_void,
                            size,
                        ))?;
                    }

                    JxlDecoderStatus_JXL_DEC_FULL_IMAGE => continue,
                    JxlDecoderStatus_JXL_DEC_SUCCESS => {
                        JxlDecoderReset(self.dec);
                        return if let Some(info) = basic_info {
                            Ok((info, buffer))
                        } else {
                            Err(JXLDecodeError::GenericError)
                        };
                    }
                    _ => return Err(JXLDecodeError::UnknownStatus(status)),
                }
            }
        }
    }
}

impl<T: PixelType> Default for JXLDecoder<T> {
    /// Create a decoder with default settings,
    //    i.e., with default memory allocator and C++ threadspool (or a naive pure Rust approach in `without-threads`).
    fn default() -> Self {
        let runner: Box<dyn JXLParallelRunner> = if cfg!(feature = "without-threads") {
            Box::new(ParallelRunner::default())
        } else {
            Box::new(ThreadsRunner::default())
        };
        Self::new(4, JXLEndianness::Native, 0, None, Some(runner))
    }
}

impl<T: PixelType> Drop for JXLDecoder<T> {
    fn drop(&mut self) {
        unsafe { JxlDecoderDestroy(self.dec) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_decode() -> Result<(), image::ImageError> {
        let sample = std::fs::read("test/sample.jxl")?;
        let mut decoder: JXLDecoder<u8> = JXLDecoder::default();
        println!(
            "Memory manager: {:#?}, Parallel Runner: {:#?}",
            decoder.memory_manager, decoder.parallel_runner
        );
        let (basic_info, buffer) = decoder.decode(&sample)?;

        assert_eq!(
            buffer.len(),
            (basic_info.xsize * basic_info.ysize * 4) as usize
        );
        Ok(())
    }

    #[test]
    fn test_rust_runner_decode() -> Result<(), Box<dyn std::error::Error>> {
        let sample = std::fs::read("test/sample.jxl")?;
        let parallel_runner = Box::new(ParallelRunner::default());

        let mut decoder: JXLDecoder<u8> =
            JXLDecoder::new(4, JXLEndianness::Native, 0, None, Some(parallel_runner));
        let parallel_buffer = decoder.decode(&sample)?;

        decoder = JXLDecoder::default();
        let single_buffer = decoder.decode(&sample)?;

        assert!(
            parallel_buffer.1 == single_buffer.1,
            "Rust runner should be the same as c++ one"
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

        let sample = std::fs::read("test/sample.jxl")?;
        let memory_manager = Box::new(MallocManager {
            layout: Layout::from_size_align(0, 8)?,
        });

        let mut decoder: JXLDecoder<u8> =
            JXLDecoder::new(4, JXLEndianness::Native, 0, Some(memory_manager), None);
        let custom_buffer = decoder.decode(&sample)?;

        decoder = JXLDecoder::default();
        let default_buffer = decoder.decode(&sample)?;

        assert!(
            custom_buffer.1 == default_buffer.1,
            "Custom memory manager should be the same as default one"
        );

        Ok(())
    }
}
