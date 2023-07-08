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

use std::ffi::c_void;

use crate::{
    cms::JxlCmsInterface, memory_manager::JxlMemoryManager, parallel_runner::JxlParallelRunner,
    JxlBasicInfo, JxlBitDepth, JxlBlendInfo, JxlBool, JxlBoxType, JxlColorEncoding,
    JxlExtraChannelInfo, JxlExtraChannelType, JxlFrameHeader, JxlPixelFormat,
};

// Opaque type
#[repr(C)]
pub struct JxlEncoder {
    _unused: [u8; 0],
}

#[deprecated(since = "0.7.0", note = "Use `JxlEncoderFrameSettings` instead")]
// Opaque type
#[repr(C)]
pub struct JxlEncoderOptions {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct JxlEncoderFrameSettings {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlEncoderStatus {
    Success = 0,
    Error = 1,
    NeedMoreOutput = 2,
    #[deprecated(
        since = "0.7.0",
        note = "JxlEncoderStatus::Error is returned with JxlEncoderError::NotSupported instead"
    )]
    NotSupported = 3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlEncoderError {
    OK = 0,
    Generic = 1,
    OutOfMemory = 2,
    Jbrd = 3, //JPEG bitstream reconstruction data could not be represented
    BadInput = 4,
    NotSupported = 0x80,
    ApiUsage = 0x81,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FrameSetting {
    Effort = 0,
    DecodingSpeed = 1,
    Resampling = 2,
    ExtraChannelResampling = 3,
    AlreadyDownsampled = 4,
    PhotonNoise = 5,
    Noise = 6,
    Dots = 7,
    Patches = 8,
    Epf = 9,
    Gaborish = 10,
    Modular = 11,
    KeepInvisible = 12,
    GroupOrder = 13,
    GroupOrderCenterX = 14,
    GroupOrderCenterY = 15,
    Responsive = 16,
    ProgressiveAc = 17,
    QprogressiveAc = 18,
    ProgressiveDc = 19,
    ChannelColorsGlobalPercent = 20,
    ChannelColorsGroupPercent = 21,
    PaletteColors = 22,
    LossyPalette = 23,
    ColorTransform = 24,
    ModularColorSpace = 25,
    ModularGroupSize = 26,
    ModularPredictor = 27,
    ModularMaTreeLearningPercent = 28,
    ModularNbPrevChannels = 29,
    JpegReconCfl = 30,
    IndexBox = 31,
    BrotliEffort = 32,
    JpegCompressBoxes = 33,
    FillEnum = 65535,
}

#[allow(deprecated)]
extern "C" {
    pub fn JxlEncoderVersion() -> u32;

    pub fn JxlEncoderCreate(memory_manager: *const JxlMemoryManager) -> *mut JxlEncoder;

    pub fn JxlEncoderReset(enc: *mut JxlEncoder);

    pub fn JxlEncoderDestroy(enc: *mut JxlEncoder);

    pub fn JxlEncoderSetCms(enc: *mut JxlEncoder, cms: JxlCmsInterface);

    pub fn JxlEncoderSetParallelRunner(
        enc: *mut JxlEncoder,
        parallel_runner: JxlParallelRunner,
        parallel_runner_opaque: *mut c_void,
    ) -> JxlEncoderStatus;

    pub fn JxlEncoderGetError(enc: *mut JxlEncoder) -> JxlEncoderError;

    pub fn JxlEncoderProcessOutput(
        enc: *mut JxlEncoder,
        next_out: *mut *mut u8,
        avail_out: *mut usize,
    ) -> JxlEncoderStatus;

    pub fn JxlEncoderSetFrameHeader(
        frame_settings: *mut JxlEncoderFrameSettings,
        frame_header: *const JxlFrameHeader,
    ) -> JxlEncoderStatus;

    pub fn JxlEncoderSetExtraChannelBlendInfo(
        frame_settings: *mut JxlEncoderFrameSettings,
        index: usize,
        blend_info: *const JxlBlendInfo,
    ) -> JxlEncoderStatus;

    pub fn JxlEncoderSetFrameName(
        frame_settings: *mut JxlEncoderFrameSettings,
        frame_name: *const u8,
    ) -> JxlEncoderStatus;

    pub fn JxlEncoderSetFrameBitDepth(
        frame_settings: *mut JxlEncoderFrameSettings,
        bit_depth: *const JxlBitDepth,
    ) -> JxlEncoderStatus;

    pub fn JxlEncoderAddJPEGFrame(
        options: *const JxlEncoderFrameSettings,
        buffer: *const u8,
        size: usize,
    ) -> JxlEncoderStatus;

    pub fn JxlEncoderAddImageFrame(
        options: *const JxlEncoderFrameSettings,
        pixel_format: *const JxlPixelFormat,
        buffer: *const c_void,
        size: usize,
    ) -> JxlEncoderStatus;

    pub fn JxlEncoderSetExtraChannelBuffer(
        frame_settings: *const JxlEncoderFrameSettings,
        pixel_format: *const JxlPixelFormat,
        buffer: *const c_void,
        size: usize,
        index: u32,
    ) -> JxlEncoderStatus;

    pub fn JxlEncoderAddBox(
        enc: *mut JxlEncoder,
        box_type: &mut JxlBoxType,
        contents: *const u8,
        size: usize,
        compress_box: JxlBool,
    ) -> JxlEncoderStatus;

    pub fn JxlEncoderUseBoxes(enc: *mut JxlEncoder) -> JxlEncoderStatus;

    pub fn JxlEncoderCloseBoxes(enc: *mut JxlEncoder);

    pub fn JxlEncoderCloseFrames(enc: *mut JxlEncoder);

    pub fn JxlEncoderCloseInput(enc: *mut JxlEncoder);

    pub fn JxlEncoderSetColorEncoding(
        enc: *mut JxlEncoder,
        color: *const JxlColorEncoding,
    ) -> JxlEncoderStatus;

    pub fn JxlEncoderSetICCProfile(
        enc: *mut JxlEncoder,
        icc_profile: *const u8,
        size: usize,
    ) -> JxlEncoderStatus;

    pub fn JxlEncoderInitBasicInfo(info: *mut JxlBasicInfo);

    pub fn JxlEncoderInitFrameHeader(frame_header: *mut JxlFrameHeader);

    pub fn JxlEncoderInitBlendInfo(blend_info: *mut JxlBlendInfo);

    pub fn JxlEncoderSetBasicInfo(
        enc: *mut JxlEncoder,
        info: *const JxlBasicInfo,
    ) -> JxlEncoderStatus;

    pub fn JxlEncoderInitExtraChannelInfo(
        channel_type: JxlExtraChannelType,
        info: *mut JxlExtraChannelInfo,
    );

    pub fn JxlEncoderSetExtraChannelInfo(
        enc: *mut JxlEncoder,
        index: usize,
        info: *const JxlExtraChannelInfo,
    ) -> JxlEncoderStatus;

    pub fn JxlEncoderSetExtraChannelName(
        enc: *mut JxlEncoder,
        index: usize,
        name: *const u8,
        size: usize,
    ) -> JxlEncoderStatus;

    pub fn JxlEncoderFrameSettingsSetOption(
        frame_settings: *mut JxlEncoderFrameSettings,
        option: FrameSetting,
        value: i64,
    ) -> JxlEncoderStatus;

    pub fn JxlEncoderFrameSettingsSetFloatOption(
        frame_settings: *mut JxlEncoderFrameSettings,
        option: FrameSetting,
        value: f32,
    ) -> JxlEncoderStatus;

    pub fn JxlEncoderUseContainer(enc: *mut JxlEncoder, use_container: bool) -> JxlEncoderStatus;

    pub fn JxlEncoderStoreJPEGMetadata(
        enc: *mut JxlEncoder,
        store_jpeg_metadata: bool,
    ) -> JxlEncoderStatus;

    pub fn JxlEncoderSetCodestreamLevel(enc: *mut JxlEncoder, level: i32) -> JxlEncoderStatus;

    pub fn JxlEncoderGetRequiredCodestreamLevel(enc: *mut JxlEncoder) -> i32;

    pub fn JxlEncoderSetFrameLossless(
        frame_settings: *mut JxlEncoderFrameSettings,
        lossless: bool,
    ) -> JxlEncoderStatus;

    #[deprecated(since = "0.7.0", note = "Use `JxlEncoderSetFrameLossless` instead")]
    pub fn JxlEncoderOptionsSetLossless(
        options: *mut JxlEncoderOptions,
        lossless: bool,
    ) -> JxlEncoderStatus;

    #[deprecated(
        since = "0.7.0",
        note = "Use `JxlEncoderFrameSettingsSetOption(frame_settings, FrameSetting::Effort, effort)` instead."
    )]
    pub fn JxlEncoderOptionsSetEffort(
        options: *mut JxlEncoderOptions,
        effort: i32,
    ) -> JxlEncoderStatus;

    #[deprecated(
        since = "0.7.0",
        note = "Use `JxlEncoderFrameSettingsSetOption(frame_settings, FrameSetting::DecodingSpeed, effort)` instead."
    )]
    pub fn JxlEncoderOptionsSetDecodingSpeed(
        options: *mut JxlEncoderOptions,
        tier: i32,
    ) -> JxlEncoderStatus;

    pub fn JxlEncoderSetFrameDistance(
        options: *mut JxlEncoderFrameSettings,
        distance: f32,
    ) -> JxlEncoderStatus;

    #[deprecated(since = "0.7.0", note = "Use `JxlEncoderSetFrameDistance` instead")]
    pub fn JxlEncoderOptionsSetDistance(
        options: *mut JxlEncoderOptions,
        distance: f32,
    ) -> JxlEncoderStatus;

    pub fn JxlEncoderFrameSettingsCreate(
        enc: *mut JxlEncoder,
        source: *const JxlEncoderFrameSettings,
    ) -> *mut JxlEncoderFrameSettings;

    #[deprecated(since = "0.7.0", note = "Use `JxlEncoderFrameSettingsCreate` instead")]
    pub fn JxlEncoderOptionsCreate(
        enc: *mut JxlEncoder,
        source: *const JxlEncoderFrameSettings,
    ) -> *mut JxlEncoderFrameSettings;

    pub fn JxlColorEncodingSetToSRGB(color_encoding: *mut JxlColorEncoding, is_gray: bool);

    pub fn JxlColorEncodingSetToLinearSRGB(color_encoding: *mut JxlColorEncoding, is_gray: bool);

    pub fn JxlEncoderAllowExpertOptions(enc: *mut JxlEncoder);
}
