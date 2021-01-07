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

#![deny(missing_docs)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
//! A safe JPEGXL Decoder wrapper.

mod error;
mod memory;
mod parallel;

#[cfg(feature = "with-image")]
mod image_support;

use error::*;
use jpegxl_sys::*;
use std::ffi::c_void;
use std::ptr::null;

pub use jpegxl_sys::JxlBasicInfo;

pub use error::JxlError;
pub use memory::*;
pub use parallel::*;

#[cfg(feature = "with-image")]
pub use image_support::*;

/// Pixel Type
/// Currently u8, u16, u32 and f32
pub trait PixelType: Clone + Default {
    /// Return the type const
    fn pixel_type() -> JxlDataType;
}
impl PixelType for u8 {
    fn pixel_type() -> JxlDataType {
        JxlDataType_JXL_TYPE_UINT8
    }
}
impl PixelType for u16 {
    fn pixel_type() -> JxlDataType {
        JxlDataType_JXL_TYPE_UINT16
    }
}
impl PixelType for u32 {
    fn pixel_type() -> JxlDataType {
        JxlDataType_JXL_TYPE_UINT32
    }
}
impl PixelType for f32 {
    fn pixel_type() -> JxlDataType {
        JxlDataType_JXL_TYPE_FLOAT
    }
}

/// JPEG XL Decoder
pub struct JxlDecoder<T: PixelType> {
    /// Opaque pointer to underlying JpegxlDecoder
    dec_ptr: *mut jpegxl_sys::JxlDecoder,
    /// Basic info about the image. `None` if it have not read the head.
    pub basic_info: Option<JxlBasicInfo>,
    /// Pixel format
    pub pixel_format: JxlPixelFormat,
    _pixel_type: std::marker::PhantomData<T>,

    /// Threadspool
    #[cfg(not(feature = "without-threads"))]
    runner: Option<ThreadsRunner>,
}

impl<T: PixelType> JxlDecoder<T> {
    /// Create a decoder.<br/>
    /// Memory manager and Parallel runner API are WIP.
    pub fn new(
        memory_manager: Option<&JxlMemoryManagerStruct>,
        parallel_runner: Option<&mut impl JXLParallelRunner>,
    ) -> Result<Self, JxlError> {
        unsafe {
            let manager_ptr = match memory_manager {
                Some(manager) => manager as *const JxlMemoryManagerStruct,
                None => null(),
            };
            let dec_ptr = JxlDecoderCreate(manager_ptr);
            if dec_ptr.is_null() {
                return Err(JxlError::CannotCreateDecoder);
            }

            if let Some(runner) = parallel_runner {
                let status = JxlDecoderSetParallelRunner(
                    dec_ptr,
                    Some(runner.runner()),
                    runner.as_opaque_ptr(),
                );
                get_error(status)?;
            }

            let pixel_format = JxlPixelFormat {
                data_type: T::pixel_type(),
                num_channels: 4,
                endianness: JxlEndianness_JXL_NATIVE_ENDIAN,
                align: 0,
            };

            #[cfg(feature = "without-threads")]
            Ok(Self {
                dec_ptr,
                basic_info: None,
                pixel_format,
                _pixel_type: std::marker::PhantomData,
            });

            #[cfg(not(feature = "without-threads"))]
            Ok(Self {
                dec_ptr,
                basic_info: None,
                pixel_format,
                _pixel_type: std::marker::PhantomData,
                runner: None,
            })
        }
    }

    /// Create a decoder with default settings,
    //    i.e., with default memory allocator and C++ threadspool (or singlethread in `without-threads`).
    pub fn default() -> Result<Self, JxlError> {
        if cfg!(feature = "without-threads") {
            Self::new(None, None::<&mut ParallelRunner>)
        } else {
            let mut runner = ThreadsRunner::default();
            let mut dec = Self::new(None, Some(&mut runner))?;
            dec.runner = Some(runner);
            Ok(dec)
        }
    }

    /// Decode a JPEG XL image.<br />
    /// Currently only support RGB(A)8/16/32 encoded static image. Color info and transformation info are discarded.
    /// # Example
    /// ```ignore
    /// # use jpegxl_rs::*;
    /// # || -> Result<(), Box<dyn std::error::Error>> {
    /// let sample = std::fs::read("test/sample.jxl")?;
    /// let mut decoder: JxlDecoder<u8> = JxlDecoder::default()?;
    /// let (info, buffer) = decoder.decode(&sample)?;
    /// # Ok(())
    /// # }();
    /// ```
    pub fn decode(&mut self, data: &[u8]) -> Result<(JxlBasicInfo, Vec<T>), JxlError> {
        let mut status: u32;

        // Stop after getting the basic info and decoding the image
        status = unsafe {
            JxlDecoderSubscribeEvents(
                self.dec_ptr,
                (JxlDecoderStatus_JXL_DEC_BASIC_INFO | JxlDecoderStatus_JXL_DEC_FULL_IMAGE) as i32,
            )
        };
        get_error(status)?;

        let next_in = &mut data.as_ptr();
        let mut avail_in = data.len() as u64;

        let mut basic_info = JxlBasicInfo::new_uninit();
        let mut buffer: Vec<T> = Vec::new();

        unsafe {
            loop {
                status = JxlDecoderProcessInput(self.dec_ptr, next_in, &mut avail_in);

                match status {
                    JxlDecoderStatus_JXL_DEC_ERROR => return Err(JxlError::GenericError),
                    JxlDecoderStatus_JXL_DEC_NEED_MORE_INPUT => {
                        return Err(JxlError::NeedMoreInput)
                    }

                    // Get the basic info
                    JxlDecoderStatus_JXL_DEC_BASIC_INFO => {
                        status = JxlDecoderGetBasicInfo(self.dec_ptr, basic_info.as_mut_ptr());
                        get_error(status)?;
                        self.basic_info = Some(basic_info.assume_init());
                    }

                    // Get the output buffer
                    JxlDecoderStatus_JXL_DEC_NEED_IMAGE_OUT_BUFFER => {
                        let mut size: u64 = 0;
                        status = JxlDecoderImageOutBufferSize(
                            self.dec_ptr,
                            &self.pixel_format,
                            &mut size,
                        );
                        get_error(status)?;

                        buffer.resize(size as usize, T::default());
                        status = JxlDecoderSetImageOutBuffer(
                            self.dec_ptr,
                            &self.pixel_format,
                            buffer.as_mut_ptr() as *mut c_void,
                            size,
                        );
                        get_error(status)?;
                    }

                    JxlDecoderStatus_JXL_DEC_FULL_IMAGE => continue,
                    JxlDecoderStatus_JXL_DEC_SUCCESS => {
                        return Ok((self.basic_info.unwrap(), buffer));
                    }
                    _ => return Err(JxlError::UnknownStatus(status)),
                }
            }
        }
    }
}

impl<T: PixelType> Drop for JxlDecoder<T> {
    fn drop(&mut self) {
        unsafe {
            JxlDecoderDestroy(self.dec_ptr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_decode() -> Result<(), image::ImageError> {
        let sample = std::fs::read("test/sample.jxl")?;
        let mut decoder: JxlDecoder<u8> = JxlDecoder::default()?;
        let (basic_info, buffer) = decoder.decode(&sample)?;

        assert_eq!(
            buffer.len(),
            (basic_info.xsize * basic_info.ysize * 4) as usize
        );
        Ok(())
    }

    #[test]
    fn test_parallel_decode() -> Result<(), Box<dyn std::error::Error>> {
        let sample = std::fs::read("test/sample.jxl")?;
        let mut parallel_runner = ParallelRunner::new();

        let mut decoder: JxlDecoder<u8> = JxlDecoder::new(None, Some(&mut parallel_runner))?;
        let parallel_buffer = decoder.decode(&sample)?;

        decoder = JxlDecoder::default()?;
        let single_buffer = decoder.decode(&sample)?;

        assert!(
            parallel_buffer.1 == single_buffer.1,
            "Multithread runner should be the same as singlethread one"
        );

        Ok(())
    }

    #[test]
    fn test_memory_manager() -> Result<(), Box<dyn std::error::Error>> {
        use memory::JXLMemoryManager;
        use std::alloc::{GlobalAlloc, Layout, System};

        struct MallocManager {
            layout: Layout,
        };

        impl JXLMemoryManager for MallocManager {
            unsafe extern "C" fn alloc(opaque: *mut c_void, size: size_t) -> *mut c_void {
                println!("Custom alloc");
                let layout = Layout::from_size_align(size as usize, 8).unwrap();
                let address = System.alloc(layout);

                let manager = (opaque as *mut Self).as_mut().unwrap();
                manager.layout = layout;

                address as *mut c_void
            }

            unsafe extern "C" fn free(opaque: *mut c_void, address: *mut c_void) {
                println!("Custom dealloc");
                let layout = (opaque as *mut Self).as_mut().unwrap().layout;
                System.dealloc(address as *mut u8, layout);
            }
        }

        let sample = std::fs::read("test/sample.jxl")?;
        let mut memory_manager = MallocManager {
            layout: Layout::from_size_align(0, 8)?,
        };

        let mut decoder: JxlDecoder<u8> = JxlDecoder::new(
            Some(&memory_manager.as_manager()),
            None::<&mut ParallelRunner>,
        )?;
        let custom_buffer = decoder.decode(&sample)?;

        decoder = JxlDecoder::default()?;
        let default_buffer = decoder.decode(&sample)?;

        assert!(
            custom_buffer.1 == default_buffer.1,
            "Custom memory manager should be the same as default one"
        );

        Ok(())
    }
}
