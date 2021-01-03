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

pub use error::JxlError;
pub use jpegxl_sys::JxlBasicInfo;
pub use memory::*;
pub use parallel::*;

#[cfg(feature = "with-image")]
pub use image_support::*;

/// JPEG XL Decoder
pub struct JxlDecoder {
    /// Opaque pointer to underlying JxlDecoder
    decoder: *mut jpegxl_sys::JxlDecoder,
    /// Basic info about the image. `None` if it have not read the head.
    pub basic_info: Option<JxlBasicInfo>,
}

impl JxlDecoder {
    /// Create a decoder.<br/>
    /// Memory manager and Parallel runner API are WIP.
    pub fn new(
        memory_manager: Option<&JxlMemoryManagerStruct>,
        parallel_runner: Option<&mut impl JpegXLParallelRunner>,
    ) -> Option<Self> {
        unsafe {
            let manager_ptr = match memory_manager {
                Some(manager) => manager as *const JxlMemoryManagerStruct,
                None => null(),
            };
            let decoder_ptr = JxlDecoderCreate(manager_ptr);
            if decoder_ptr.is_null() {
                return None;
            }

            if let Some(runner) = parallel_runner {
                let status = JxlDecoderSetParallelRunner(
                    decoder_ptr,
                    Some(runner.runner()),
                    runner.as_opaque_ptr(),
                );
                get_error(status).ok()?;
            }

            Some(JxlDecoder {
                decoder: decoder_ptr,
                basic_info: None,
            })
        }
    }

    /// Create a decoder with default settings, e.g. with default memory allocator and single-threaded.
    pub fn new_with_default() -> Option<Self> {
        Self::new(None, None::<&mut ParallelRunner>)
    }

    // TODO: Handle more data types when the underlying library implemented them
    fn get_pixel_format(&self) -> JxlPixelFormat {
        JxlPixelFormat {
            data_type: JxlDataType_JXL_TYPE_UINT8,
            num_channels: if self.basic_info.unwrap().alpha_bits == 0 {
                3
            } else {
                4
            },
            endianness: JxlEndianness_JXL_NATIVE_ENDIAN,
            align: 0
        }
    }

    /// Decode a JPEG XL image.<br />
    /// Currently only support RGB(A)8 encoded static image. Color info and transformation info are discarded.
    /// # Example
    /// ```
    /// # use jpegxl_rs::*;
    /// # || -> Result<(), Box<dyn std::error::Error>> {
    /// let sample = std::fs::read("test/sample.jxl")?;
    /// let mut decoder = JxlDecoder::new_with_default().ok_or(JxlError::CannotCreateDecoder)?;
    /// let buffer = decoder.decode(&sample)?;
    /// # Ok(())
    /// # }();
    /// ```
    pub fn decode<T: AsRef<[u8]>>(&mut self, data: &T) -> Result<Vec<u8>, JxlError> {
        let mut status;
        status = unsafe {
            JxlDecoderSubscribeEvents(
                self.decoder,
                (JxlDecoderStatus_JXL_DEC_BASIC_INFO
                    | JxlDecoderStatus_JXL_DEC_FULL_IMAGE) as i32,
            )
        };
        get_error(status)?;

        let next_in = &mut data.as_ref().as_ptr();
        let mut avail_in = data.as_ref().len() as size_t;
        status = unsafe { JxlDecoderProcessInput(self.decoder, next_in, &mut avail_in) };
        get_error(status)?;

        let mut basic_info = JxlBasicInfo::new_uninit();
        unsafe {
            status = JxlDecoderGetBasicInfo(self.decoder, basic_info.as_mut_ptr());
            get_error(status)?;
            let basic_info = basic_info.assume_init();
            self.basic_info = Some(basic_info);
        }

        status = unsafe { JxlDecoderProcessInput(self.decoder, next_in, &mut avail_in) };
        get_error(status)?;

        // Get the buffer size
        let mut size: size_t = 0;
        let pixel_format = self.get_pixel_format();
        status = unsafe { JxlDecoderImageOutBufferSize(self.decoder, &pixel_format, &mut size) };
        get_error(status)?;

        // Create a buffer to hold decoded image
        let mut buffer: Vec<u8> = Vec::with_capacity(size as usize);
        unsafe {
            buffer.set_len(size as usize);
            status = JxlDecoderSetImageOutBuffer(
                self.decoder,
                &pixel_format,
                buffer.as_mut_ptr() as *mut c_void,
                size,
            );
            get_error(status)?;
        }

        // Read what left of the image
        status = unsafe { JxlDecoderProcessInput(self.decoder, next_in, &mut avail_in) };
        get_error(status)?;

        Ok(buffer)
    }
}

impl Drop for JxlDecoder {
    fn drop(&mut self) {
        unsafe {
            JxlDecoderDestroy(self.decoder);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_decode() -> Result<(), image::ImageError> {
        let sample = std::fs::read("test/sample.jxl")?;
        let mut decoder =
            JxlDecoder::new_with_default().ok_or(JxlError::CannotCreateDecoder)?;
        let buffer = decoder.decode(&sample)?;
        let basic_info = decoder.basic_info.unwrap();

        use image::{ImageBuffer, RgbImage};
        let image: RgbImage =
            ImageBuffer::from_raw(basic_info.xsize, basic_info.ysize, buffer).unwrap();

        image.save("sample.png")?;
        std::fs::remove_file("sample.png")?;

        Ok(())
    }

    #[test]
    fn test_parallel_decode() -> Result<(), Box<dyn std::error::Error>> {
        let sample = std::fs::read("test/sample.jxl")?;
        let mut parallel_runner = ParallelRunner::new();

        let mut decoder = JxlDecoder::new(None, Some(&mut parallel_runner))
            .ok_or(JxlError::CannotCreateDecoder)?;
        let parallel_buffer = decoder.decode(&sample)?;

        decoder = JxlDecoder::new_with_default().ok_or(JxlError::CannotCreateDecoder)?;
        let single_buffer = decoder.decode(&sample)?;

        assert!(
            parallel_buffer == single_buffer,
            "Multithread runner should be the same as singlethread one"
        );

        Ok(())
    }

    #[test]
    fn test_memory_manager() -> Result<(), Box<dyn std::error::Error>> {
        use memory::JxlMemoryManager;
        use std::alloc::{GlobalAlloc, Layout, System};

        struct MallocManager {
            layout: Layout,
        };

        impl JxlMemoryManager for MallocManager {
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
        let memory_manager = (MallocManager {
            layout: Layout::from_size_align(0, 8)?,
        })
        .to_manager();

        let mut decoder = JxlDecoder::new(Some(&memory_manager), None::<&mut ParallelRunner>)
            .ok_or(JxlError::CannotCreateDecoder)?;
        let custom_buffer = decoder.decode(&sample)?;

        decoder = JxlDecoder::new_with_default().ok_or(JxlError::CannotCreateDecoder)?;
        let default_buffer = decoder.decode(&sample)?;

        assert!(
            custom_buffer == default_buffer,
            "Custom memory manager should be the same as default one"
        );

        Ok(())
    }
}
