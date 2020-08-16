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

pub struct JpegxlDecoder {
    /// Opaque pointer to underlying JpegxlDecoder
    decoder: *mut jpegxl_sys::JpegxlDecoder,
    /// Basic info about the image. `None` if it have not read the head.
    basic_info: Option<JpegxlBasicInfo>,
}

impl JpegxlDecoder {
    // TODO: Add memory manager API
    // TODO: Add parallel runner API
    pub fn new(
        manager: *const JpegxlMemoryManager,
        runner: JpegxlParallelRunner,
        runner_ptr: *mut std::ffi::c_void,
    ) -> Option<Self> {
        unsafe {
            let decoder_ptr = JpegxlDecoderCreate(manager);
            if decoder_ptr.is_null() {
                return None;
            }
            let status = JpegxlDecoderSetParallelRunner(decoder_ptr, runner, runner_ptr);
            get_error(status).ok()?;

            Some(JpegxlDecoder {
                decoder: decoder_ptr,
                basic_info: None,
            })
        }
    }

    pub fn new_with_default() -> Option<Self> {
        Self::new(null(), None, null_mut())
    }

    // TODO: Handle more data types when the underlying library implemented them
    fn get_pixel_format(&self, basic_info: &JpegxlBasicInfo) -> JpegxlPixelFormat {
        JpegxlPixelFormat {
            data_type: JpegxlDataType_JPEGXL_TYPE_UINT8,
            num_channels: if basic_info.alpha_bits == 0 { 3 } else { 4 },
        }
    }

    pub fn decode(&mut self, data: &[u8]) -> Result<Vec<u8>, JpegxlError> {
        unsafe {
            let mut status;
            status = JpegxlDecoderSubscribeEvents(
                self.decoder,
                (JpegxlDecoderStatus_JPEGXL_DEC_BASIC_INFO
                    | JpegxlDecoderStatus_JPEGXL_DEC_FULL_IMAGE) as i32,
            );
            get_error(status)?;

            let next_in = &mut data.as_ptr();
            let mut avail_in = data.len() as size_t;
            status = JpegxlDecoderProcessInput(self.decoder, next_in, &mut avail_in);
            get_error(status)?;

            let mut basic_info = JpegxlBasicInfo::new_uninit();
            status = JpegxlDecoderGetBasicInfo(self.decoder, basic_info.as_mut_ptr());
            get_error(status)?;
            let basic_info = basic_info.assume_init();
            self.basic_info = Some(basic_info);

            // Get the buffer size
            let mut size: size_t = 0;
            let pixel_format = self.get_pixel_format(&basic_info);
            status = JpegxlDecoderImageOutBufferSize(self.decoder, &pixel_format, &mut size);
            get_error(status)?;

            // Create a buffer to hold decoded image
            let mut buffer: Vec<u8> = Vec::with_capacity(size as usize);
            buffer.set_len(size as usize);
            status = JpegxlDecoderSetImageOutBuffer(
                self.decoder,
                &pixel_format,
                buffer.as_mut_ptr() as *mut std::ffi::c_void,
                size,
            );
            get_error(status)?;

            // Read what left of the image
            status = JpegxlDecoderProcessInput(self.decoder, next_in, &mut avail_in);
            get_error(status)?;

            Ok(buffer)
        }
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
        let buffer = decoder.decode(sample.as_slice())?;
        let basic_info = decoder.basic_info.unwrap();

        use image::{ImageBuffer, RgbImage};
        let image: RgbImage =
            ImageBuffer::from_raw(basic_info.xsize, basic_info.ysize, buffer).unwrap();

        image.save("sample.png")?;
        std::fs::remove_file("sample.png")?;

        Ok(())
    }
}
