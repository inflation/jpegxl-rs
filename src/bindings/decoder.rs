/*
This file is part of jpegxl-sys.

jpegxl-sys is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

jpegxl-sys is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with jpegxl-sys.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::{
    ffi::c_void,
    os::raw::{c_char, c_int},
};

use crate::common::{
    JxlBasicInfo, JxlColorEncoding, JxlColorProfileTarget, JxlExtraChannelInfo, JxlFrameHeader,
    JxlMemoryManager, JxlParallelRunner, JxlPixelFormat, JxlSignature,
};

// Opaque type
#[allow(clippy::module_name_repetitions)]
#[repr(C)]
pub struct JxlDecoder {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum JxlDecoderStatus {
    Success = 0,
    Error = 1,
    NeedMoreInput = 2,
    NeedPreviewOutBuffer = 3,
    NeedDcOutBuffer = 4,
    NeedImageOutBuffer = 5,
    JpegNeedMoreOutput = 6,
    BasicInfo = 0x40,
    Extensions = 0x80,
    ColorEncoding = 0x100,
    PreviewImage = 0x200,
    Frame = 0x400,
    DcImage = 0x800,
    FullImage = 0x1000,
    JpegReconstruction = 0x2000,
}

#[macro_export]
macro_rules! jxl_dec_events {
    ( $( $x: expr ),* ) => {
        {
            let mut tmp = 0;
            $(
                tmp |= $x as i32;
            )*
            tmp
        }
    };
}

pub type JxlImageOutCallback = extern "C" fn(
    opaque: *mut c_void,
    x: usize,
    y: usize,
    num_pixels: usize,
    pixels: *const c_void,
);

extern "C" {
    pub fn JxlSignatureCheck(buf: *const u8, len: usize) -> JxlSignature;
    pub fn JxlDecoderCreate(memory_manager: *const JxlMemoryManager) -> *mut JxlDecoder;
    pub fn JxlDecoderReset(dec: *mut JxlDecoder);
    pub fn JxlDecoderDestroy(dec: *mut JxlDecoder);
    pub fn JxlDecoderVersion() -> u32;
    pub fn JxlDecoderRewind(dec: *mut JxlDecoder);
    pub fn JxlDecoderSkipFrames(dec: *mut JxlDecoder, amount: usize);

    pub fn JxlDecoderDefaultPixelFormat(
        dec: *const JxlDecoder,
        format: *mut JxlPixelFormat,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderSetParallelRunner(
        dec: *mut JxlDecoder,
        parallel_runner: JxlParallelRunner,
        parallel_runner_opaque: *mut c_void,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderSizeHintBasicInfo(dec: *const JxlDecoder) -> usize;

    pub fn JxlDecoderSubscribeEvents(
        dec: *mut JxlDecoder,
        events_wanted: c_int,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderSetKeepOrientation(
        dec: *mut JxlDecoder,
        keep_orientation: bool,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderProcessInput(dec: *mut JxlDecoder) -> JxlDecoderStatus;

    pub fn JxlDecoderSetInput(
        dec: *mut JxlDecoder,
        data: *const u8,
        size: usize,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderReleaseInput(dec: *mut JxlDecoder) -> usize;

    pub fn JxlDecoderGetBasicInfo(
        dec: *const JxlDecoder,
        info: *mut JxlBasicInfo,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderGetExtraChannelInfo(
        dec: *const JxlDecoder,
        index: usize,
        info: *mut JxlExtraChannelInfo,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderGetExtraChannelName(
        dec: *const JxlDecoder,
        index: usize,
        name: *mut c_char,
        size: usize,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderGetColorAsEncodedProfile(
        dec: *const JxlDecoder,
        format: *const JxlPixelFormat,
        target: JxlColorProfileTarget,
        color_encoding: *mut JxlColorEncoding,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderGetICCProfileSize(
        dec: *const JxlDecoder,
        format: *const JxlPixelFormat,
        target: JxlColorProfileTarget,
        size: *mut usize,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderGetColorAsICCProfile(
        dec: *const JxlDecoder,
        format: *const JxlPixelFormat,
        target: JxlColorProfileTarget,
        icc_profile: *mut u8,
        size: usize,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderSetPreferredColorProfile(
        dec: *mut JxlDecoder,
        color_encoding: *const JxlColorEncoding,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderPreviewOutBufferSize(
        dec: *const JxlDecoder,
        format: *const JxlPixelFormat,
        size: *mut usize,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderSetPreviewOutBuffer(
        dec: *mut JxlDecoder,
        format: *const JxlPixelFormat,
        buffer: *mut c_void,
        size: usize,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderGetFrameHeader(
        dec: *const JxlDecoder,
        header: *mut JxlFrameHeader,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderGetFrameName(
        dec: *const JxlDecoder,
        name: *mut c_char,
        size: usize,
    ) -> JxlDecoderStatus;

    #[deprecated(
        since = "0.5.0",
        note = "please use `JxlDecoderSetImageOutCallback` instead"
    )]
    pub fn JxlDecoderDCOutBufferSize(
        dec: *const JxlDecoder,
        format: *const JxlPixelFormat,
        size: *mut usize,
    ) -> JxlDecoderStatus;

    #[deprecated(
        since = "0.5.0",
        note = "please use `JxlDecoderSetImageOutCallback` instead"
    )]
    pub fn JxlDecoderSetDCOutBuffer(
        dec: *mut JxlDecoder,
        format: *const JxlPixelFormat,
        buffer: *mut c_void,
        size: usize,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderImageOutBufferSize(
        dec: *const JxlDecoder,
        format: *const JxlPixelFormat,
        size: *mut usize,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderSetImageOutBuffer(
        dec: *mut JxlDecoder,
        format: *const JxlPixelFormat,
        buffer: *mut c_void,
        size: usize,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderSetImageOutCallback(
        dec: *mut JxlDecoder,
        format: *const JxlPixelFormat,
        callback: JxlImageOutCallback,
        opaque: *mut c_void,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderExtraChannelBufferSize(
        dec: *const JxlDecoder,
        format: *const JxlPixelFormat,
        size: *mut usize,
        index: u32,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderSetExtraChannelBuffer(
        dec: *mut JxlDecoder,
        format: *const JxlPixelFormat,
        buffer: *mut c_void,
        size: usize,
        index: u32,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderSetJPEGBuffer(
        dec: *mut JxlDecoder,
        data: *mut u8,
        size: usize,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderReleaseJPEGBuffer(dec: *mut JxlDecoder) -> usize;

    pub fn JxlDecoderFlushImage(dec: *mut JxlDecoder) -> JxlDecoderStatus;
}
