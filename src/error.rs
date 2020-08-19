#![allow(non_upper_case_globals)]

use jpegxl_sys::*;

/// Errors derived from JpegxlDecoderStatus
#[derive(Debug)]
pub enum JpegxlError {
    /// Cannot create a decoder
    CannotCreateDecoder,
    /// Unknown Error
    // TODO: underlying library is working on a way to retrieve error message
    GenericError,
    /// Need more input bytes
    NeedMoreInput,
    /// Need more output buffer
    NeedMoreOutput,
}

impl std::fmt::Display for JpegxlError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for JpegxlError {}

#[cfg(any(features = "with-image", test))]
impl From<JpegxlError> for image::ImageError {
    fn from(e: JpegxlError) -> Self {
        use image::error::{DecodingError, ImageFormatHint};
        image::ImageError::Decoding(DecodingError::new(
            ImageFormatHint::Name("JPEGXL".to_string()),
            e,
        ))
    }
}

/// Error mapping from underlying C const to JpegxlError enum
pub fn get_error(status: JpegxlDecoderStatus) -> Result<(), JpegxlError> {
    match status {
        JpegxlDecoderStatus_JPEGXL_DEC_ERROR => Err(JpegxlError::GenericError),
        JpegxlDecoderStatus_JPEGXL_DEC_NEED_MORE_INPUT => Err(JpegxlError::NeedMoreInput),
        JpegxlDecoderStatus_JPEGXL_DEC_NEED_MORE_OUTPUT => Err(JpegxlError::NeedMoreOutput),
        _ => Ok(()),
    }
}
