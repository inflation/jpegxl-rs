use jpegxl_sys::{
    JxlColorProfileTarget, JxlDecoder, JxlDecoderStatus, JxlEncoder, JxlEncoderOptions,
    JxlEncoderStatus, JxlPixelFormat,
};
use std::os::raw::c_void;

/// Masking C functions with better signature
extern "C" {
    pub(crate) fn JxlDecoderSubscribeEvents(
        dec: *mut JxlDecoder,
        events_wanted: JxlDecoderStatus,
    ) -> JxlDecoderStatus;

    pub(crate) fn JxlDecoderImageOutBufferSize(
        dec: *const JxlDecoder,
        format: &JxlPixelFormat,
        size: &mut usize,
    ) -> JxlDecoderStatus;

    pub(crate) fn JxlDecoderSetImageOutBuffer(
        dec: *mut JxlDecoder,
        format: &JxlPixelFormat,
        buffer: *mut c_void,
        size: usize,
    ) -> JxlDecoderStatus;

    pub(crate) fn JxlDecoderGetICCProfileSize(
        dec: *const JxlDecoder,
        format: &JxlPixelFormat,
        target: JxlColorProfileTarget,
        size: &mut usize,
    ) -> JxlDecoderStatus;

    pub(crate) fn JxlDecoderGetColorAsICCProfile(
        dec: *const JxlDecoder,
        format: &JxlPixelFormat,
        target: JxlColorProfileTarget,
        icc_profile: *mut u8,
        size: usize,
    ) -> JxlDecoderStatus;

    #[allow(dead_code)] // TODO: Upstream not implemented
    pub(crate) fn JxlDecoderSetJPEGBuffer(
        dec: *mut JxlDecoder,
        data: *mut u8,
        size: usize,
    ) -> JxlDecoderStatus;

    // Encoder

    pub(crate) fn JxlEncoderProcessOutput(
        enc: *mut JxlEncoder,
        next_out: *mut *mut u8,
        avail_out: &mut usize,
    ) -> JxlEncoderStatus;

    pub(crate) fn JxlEncoderAddJPEGFrame(
        options: *const JxlEncoderOptions,
        buffer: *const u8,
        size: usize,
    ) -> JxlEncoderStatus;

    pub(crate) fn JxlEncoderAddImageFrame(
        options: *const JxlEncoderOptions,
        pixel_format: &JxlPixelFormat,
        buffer: *const u8,
        size: usize,
    ) -> JxlEncoderStatus;

}
