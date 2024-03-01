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

use crate::{
    cms::JxlCmsInterface,
    codestream_header::{JxlBasicInfo, JxlBlendInfo, JxlExtraChannelInfo, JxlFrameHeader},
    color_encoding::JxlColorEncoding,
    memory_manager::JxlMemoryManager,
    parallel_runner::JxlParallelRunner,
    types::{JxlBitDepth, JxlBool, JxlBoxType, JxlPixelFormat},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlSignature {
    NotEnoughBytes = 0,
    Invalid = 1,
    Codestream = 2,
    Container = 3,
}

// Opaque type
#[repr(C)]
pub struct JxlDecoder {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlDecoderStatus {
    Success = 0,
    Error = 1,
    NeedMoreInput = 2,
    NeedPreviewOutBuffer = 3,
    NeedImageOutBuffer = 5,
    JpegNeedMoreOutput = 6,
    BoxNeedMoreOutput = 7,
    BasicInfo = 0x40,
    ColorEncoding = 0x100,
    PreviewImage = 0x200,
    Frame = 0x400,
    FullImage = 0x1000,
    JpegReconstruction = 0x2000,
    Box = 0x4000,
    FrameProgression = 0x8000,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlProgressiveDetail {
    Frames = 0,
    DC = 1,
    LastPasses = 2,
    Passes = 3,
    DCProgressive = 4,
    DCGroups = 5,
    Groups = 6,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlColorProfileTarget {
    Original = 0,
    Data = 1,
}

pub type JxlImageOutCallback = extern "C" fn(
    opaque: *mut c_void,
    x: usize,
    y: usize,
    num_pixels: usize,
    pixels: *const c_void,
);

pub type JxlImageOutInitCallback = extern "C" fn(
    init_opaque: *mut c_void,
    num_threads: usize,
    num_pixels_per_thread: usize,
) -> *mut c_void;

pub type JxlImageOutRunCallback = extern "C" fn(
    run_opaque: *mut c_void,
    thread_id: usize,
    x: usize,
    y: usize,
    num_pixels: usize,
    pixels: *const c_void,
);

pub type JxlImageOutDestroyCallback = extern "C" fn(run_opaque: *mut c_void);

extern "C" {
    pub fn JxlDecoderVersion() -> u32;
    pub fn JxlSignatureCheck(buf: *const u8, len: usize) -> JxlSignature;
    pub fn JxlDecoderCreate(memory_manager: *const JxlMemoryManager) -> *mut JxlDecoder;
    pub fn JxlDecoderReset(dec: *mut JxlDecoder);
    pub fn JxlDecoderDestroy(dec: *mut JxlDecoder);
    pub fn JxlDecoderRewind(dec: *mut JxlDecoder);
    pub fn JxlDecoderSkipFrames(dec: *mut JxlDecoder, amount: usize);

    pub fn JxlDecoderSkipCurrentFrame(dec: *mut JxlDecoder) -> JxlDecoderStatus;

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
        keep_orientation: JxlBool,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderSetUnpremultiplyAlpha(
        dec: *mut JxlDecoder,
        unpremul_alpha: JxlBool,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderSetRenderSpotcolors(
        dec: *mut JxlDecoder,
        render_spotcolors: JxlBool,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderSetCoalescing(dec: *mut JxlDecoder, coalescing: JxlBool) -> JxlDecoderStatus;

    pub fn JxlDecoderProcessInput(dec: *mut JxlDecoder) -> JxlDecoderStatus;

    pub fn JxlDecoderSetInput(
        dec: *mut JxlDecoder,
        data: *const u8,
        size: usize,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderReleaseInput(dec: *mut JxlDecoder) -> usize;

    pub fn JxlDecoderCloseInput(dec: *mut JxlDecoder);

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
        target: JxlColorProfileTarget,
        color_encoding: *mut JxlColorEncoding,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderGetICCProfileSize(
        dec: *const JxlDecoder,
        target: JxlColorProfileTarget,
        size: *mut usize,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderGetColorAsICCProfile(
        dec: *const JxlDecoder,
        target: JxlColorProfileTarget,
        icc_profile: *mut u8,
        size: usize,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderSetPreferredColorProfile(
        dec: *mut JxlDecoder,
        color_encoding: *const JxlColorEncoding,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderSetDesiredIntensityTarget(
        dec: *mut JxlDecoder,
        desired_intensity_target: f32,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderSetOutputColorProfile(
        dec: *mut JxlDecoder,
        color_encoding: *const JxlColorEncoding,
        icc_data: *const u8,
        icc_size: usize,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderSetCms(dec: *mut JxlDecoder, cms: JxlCmsInterface) -> JxlDecoderStatus;

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

    pub fn JxlDecoderGetExtraChannelBlendInfo(
        dec: *const JxlDecoder,
        index: usize,
        blend_info: *mut JxlBlendInfo,
    );

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

    pub fn JxlDecoderSetMultithreadedImageOutCallback(
        dec: *mut JxlDecoder,
        format: *const JxlPixelFormat,
        init_callback: JxlImageOutInitCallback,
        run_callback: JxlImageOutRunCallback,
        destroy_callback: JxlImageOutDestroyCallback,
        init_opaque: *mut c_void,
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

    pub fn JxlDecoderSetBoxBuffer(
        dec: *mut JxlDecoder,
        data: *mut u8,
        size: usize,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderReleaseBoxBuffer(dec: *mut JxlDecoder) -> usize;

    pub fn JxlDecoderSetDecompressBoxes(
        dec: *mut JxlDecoder,
        decompress: JxlBool,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderGetBoxType(
        dec: *mut JxlDecoder,
        box_type: &mut JxlBoxType,
        decompressed: JxlBool,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderGetBoxSizeRaw(dec: *mut JxlDecoder, size: *mut u64) -> JxlDecoderStatus;

    pub fn JxlDecoderGetBoxSizeContents(dec: *mut JxlDecoder, size: *mut u64) -> JxlDecoderStatus;

    pub fn JxlDecoderSetProgressiveDetail(
        dec: *mut JxlDecoder,
        detail: JxlProgressiveDetail,
    ) -> JxlDecoderStatus;

    pub fn JxlDecoderGetIntendedDownsamplingRatio(dec: *const JxlDecoder) -> usize;

    pub fn JxlDecoderFlushImage(dec: *mut JxlDecoder) -> JxlDecoderStatus;

    pub fn JxlDecoderSetImageOutBitDepth(
        dec: *mut JxlDecoder,
        bit_depth: *const JxlBitDepth,
    ) -> JxlDecoderStatus;
}
