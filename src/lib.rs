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

#[cfg(features = "with-image")]
mod image_support;

use error::*;
use jpegxl_sys::*;
use std::ffi::c_void;
use std::ptr::null;

pub use error::JpegxlError;
pub use jpegxl_sys::JpegxlBasicInfo;
pub use memory::*;
pub use parallel::*;

#[cfg(features = "with-image")]
pub use image_support::*;

/// JPEG XL Decoder
pub struct JpegxlDecoder {
    /// Opaque pointer to underlying JpegxlDecoder
    decoder: *mut jpegxl_sys::JpegxlDecoder,
    /// Basic info about the image. `None` if it have not read the head.
    pub basic_info: Option<JpegxlBasicInfo>,
}

impl JpegxlDecoder {
    /// Create a decoder.<br/>
    /// Memory manager and Parallel runner API are WIP.
    pub fn new(
        memory_manager: Option<&JpegxlMemoryManagerStruct>,
        parallel_runner: Option<&mut impl JpegXLParallelRunner>,
    ) -> Option<Self> {
        unsafe {
            let manager_ptr = match memory_manager {
                Some(manager) => manager as *const JpegxlMemoryManagerStruct,
                None => null(),
            };
            let decoder_ptr = JpegxlDecoderCreate(manager_ptr);
            if decoder_ptr.is_null() {
                return None;
            }

            if let Some(runner) = parallel_runner {
                let status = JpegxlDecoderSetParallelRunner(
                    decoder_ptr,
                    Some(runner.runner()),
                    runner.as_opaque_ptr(),
                );
                get_error(status).ok()?;
            }

            Some(JpegxlDecoder {
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
    fn get_pixel_format(&self) -> JpegxlPixelFormat {
        JpegxlPixelFormat {
            data_type: JpegxlDataType_JPEGXL_TYPE_UINT8,
            num_channels: if self.basic_info.unwrap().alpha_bits == 0 {
                3
            } else {
                4
            },
        }
    }

    /// Decode a JPEG XL image.<br />
    /// Currently only support RGB(A)8 encoded static image. Color info and transformation info are discarded.
    /// # Example
    /// ```
    /// # use jpegxl_rs::*;
    /// # || -> Result<(), Box<dyn std::error::Error>> {
    /// let sample = std::fs::read("test/sample.jxl")?;
    /// let mut decoder = JpegxlDecoder::new_with_default().ok_or(JpegxlError::CannotCreateDecoder)?;
    /// let buffer = decoder.decode(&sample)?;
    /// # Ok(())
    /// # }();
    /// ```
    pub fn decode<T: AsRef<[u8]>>(&mut self, data: &T) -> Result<Vec<u8>, JpegxlError> {
        let mut status;
        status = unsafe {
            JpegxlDecoderSubscribeEvents(
                self.decoder,
                (JpegxlDecoderStatus_JPEGXL_DEC_BASIC_INFO
                    | JpegxlDecoderStatus_JPEGXL_DEC_FULL_IMAGE) as i32,
            )
        };
        get_error(status)?;

        let next_in = &mut data.as_ref().as_ptr();
        let mut avail_in = data.as_ref().len() as size_t;
        status = unsafe { JpegxlDecoderProcessInput(self.decoder, next_in, &mut avail_in) };
        get_error(status)?;

        let mut basic_info = JpegxlBasicInfo::new_uninit();
        unsafe {
            status = JpegxlDecoderGetBasicInfo(self.decoder, basic_info.as_mut_ptr());
            get_error(status)?;
            let basic_info = basic_info.assume_init();
            self.basic_info = Some(basic_info);
        }

        // Get the buffer size
        let mut size: size_t = 0;
        let pixel_format = self.get_pixel_format();
        status = unsafe { JpegxlDecoderImageOutBufferSize(self.decoder, &pixel_format, &mut size) };
        get_error(status)?;

        // Create a buffer to hold decoded image
        let mut buffer: Vec<u8> = Vec::with_capacity(size as usize);
        unsafe {
            buffer.set_len(size as usize);
            status = JpegxlDecoderSetImageOutBuffer(
                self.decoder,
                &pixel_format,
                buffer.as_mut_ptr() as *mut c_void,
                size,
            );
            get_error(status)?;
        }

        // Read what left of the image
        status = unsafe { JpegxlDecoderProcessInput(self.decoder, next_in, &mut avail_in) };
        get_error(status)?;

        Ok(buffer)
    }
}

impl Drop for JpegxlDecoder {
    fn drop(&mut self) {
        unsafe {
            JpegxlDecoderDestroy(self.decoder);
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
            JpegxlDecoder::new_with_default().ok_or(JpegxlError::CannotCreateDecoder)?;
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

        let mut decoder = JpegxlDecoder::new(None, Some(&mut parallel_runner))
            .ok_or(JpegxlError::CannotCreateDecoder)?;
        let parallel_buffer = decoder.decode(&sample)?;

        decoder = JpegxlDecoder::new_with_default().ok_or(JpegxlError::CannotCreateDecoder)?;
        let single_buffer = decoder.decode(&sample)?;

        assert!(
            parallel_buffer == single_buffer,
            "Multithread runner should be the same as singlethread one"
        );

        Ok(())
    }

    #[test]
    fn test_memory_manager() -> Result<(), Box<dyn std::error::Error>> {
        use memory::JpegxlMemoryManager;
        use std::alloc::{GlobalAlloc, Layout, System};
        use std::collections::HashMap;

        struct MallocManager {
            table: HashMap<usize, Layout>,
        };

        impl JpegxlMemoryManager for MallocManager {
            unsafe extern "C" fn alloc(opaque: *mut c_void, size: size_t) -> *mut c_void {
                let layout = Layout::from_size_align(size as usize, 8).unwrap();
                let address = System.alloc(layout);

                let manager = (opaque as *mut Self).as_mut().unwrap();
                manager.table.insert(address as usize, layout);

                address as *mut c_void
            }

            unsafe extern "C" fn free(opaque: *mut c_void, address: *mut c_void) {
                let layout = (opaque as *mut Self)
                    .as_mut()
                    .unwrap()
                    .table
                    .get(&(address as usize))
                    .unwrap();
                System.dealloc(address as *mut u8, *layout);
            }
        }

        let sample = std::fs::read("test/sample.jxl")?;
        let memory_manager = (MallocManager {
            table: HashMap::new(),
        })
        .new();

        let mut decoder = JpegxlDecoder::new(Some(&memory_manager), None::<&mut ParallelRunner>)
            .ok_or(JpegxlError::CannotCreateDecoder)?;
        let custom_buffer = decoder.decode(&sample)?;

        decoder = JpegxlDecoder::new_with_default().ok_or(JpegxlError::CannotCreateDecoder)?;
        let default_buffer = decoder.decode(&sample)?;

        assert!(
            custom_buffer == default_buffer,
            "Custom memory manager should be the same as default one"
        );

        Ok(())
    }
}
