#![deny(missing_docs)]
//! A safe JPEGXL Decoder wrapper.

mod error;
#[cfg(features = "with-image")]
mod image_support;

use error::*;
use jpegxl_sys::*;
use std::ptr::{null, null_mut};

pub use error::JpegxlError;
pub use jpegxl_sys::JpegxlBasicInfo;

#[cfg(features = "with-image")]
pub use image_support::*;

/// JPEG XL Decoder
pub struct JpegxlDecoder {
    /// Opaque pointer to underlying JpegxlDecoder
    decoder: *mut jpegxl_sys::JpegxlDecoder,
    /// Basic info about the image. `None` if it have not read the head.
    basic_info: Option<JpegxlBasicInfo>,
}

impl JpegxlDecoder {
    /// Create a decoder.<br/>
    /// Memory manager and Parallel runner API are WIP.
    // TODO: Add memory manager API
    // TODO: Add parallel runner API
    pub fn new(
        manager: *const JpegxlMemoryManager,
        runner_func: JpegxlParallelRunner,
        runner_ptr: *mut std::ffi::c_void,
    ) -> Option<Self> {
        unsafe {
            let decoder_ptr = JpegxlDecoderCreate(manager);
            if decoder_ptr.is_null() {
                return None;
            }
            let status = JpegxlDecoderSetParallelRunner(decoder_ptr, runner_func, runner_ptr);
            get_error(status).ok()?;

            Some(JpegxlDecoder {
                decoder: decoder_ptr,
                basic_info: None,
            })
        }
    }

    /// Create a decoder with default settings, e.g. with default memory allocator and single-threaded.
    pub fn new_with_default() -> Option<Self> {
        Self::new(null(), None, null_mut())
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
                buffer.as_mut_ptr() as *mut std::ffi::c_void,
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
    use super::{JpegxlDecoder, JpegxlError};
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
}
