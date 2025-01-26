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

//! Definitions of structs and enums for the metadata from the JPEG XL
//! codestream headers (signature, metadata, preview dimensions, ...), excluding
//! color encoding which is in [`crate::color::color_encoding`].

use crate::common::types::JxlBool;

#[cfg(doc)]
use crate::{
    color::color_encoding::{JxlColorEncoding, JxlColorSpace},
    decode::{
        JxlColorProfileTarget, JxlDecoderGetExtraChannelBlendInfo, JxlDecoderGetExtraChannelInfo,
    },
    encoder::encode::{JxlEncoderCloseFrames, JxlEncoderSetFrameName},
};

/// Image orientation metadata.
///
/// Values 1..8 match the EXIF definitions.
/// The name indicates the operation to perform to transform from the encoded
/// image to the display image.
#[repr(u32)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(feature="num-enum", derive(num_enum::TryFromPrimitive))]
pub enum JxlOrientation {
    Identity = 1,
    FlipHorizontal = 2,
    Rotate180 = 3,
    FlipVertical = 4,
    Transpose = 5,
    Rotate90Cw = 6,
    AntiTranspose = 7,
    Rotate90Ccw = 8,
}

/// Given type of an extra channel.
#[repr(C)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum JxlExtraChannelType {
    Alpha,
    Depth,
    SpotColor,
    SelectionMask,
    Black,
    Cfa,
    Thermal,
    Reserved0,
    Reserved1,
    Reserved2,
    Reserved3,
    Reserved4,
    Reserved5,
    Reserved6,
    Reserved7,
    Unknown,
    Optional,
}

/// The codestream preview header
#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlPreviewHeader {
    /// Preview width in pixels
    pub xsize: u32,
    /// Preview height in pixels
    pub ysize: u32,
}

/// The codestream animation header, optionally present in the beginning of
/// the codestream, and if it is it applies to all animation frames, unlike
/// [`JxlFrameHeader`] which applies to an individual frame.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlAnimationHeader {
    /// Numerator of ticks per second of a single animation frame time unit
    pub tps_numerator: u32,

    /// Denominator of ticks per second of a single animation frame time unit
    pub tps_denominator: u32,

    /// Amount of animation loops, or 0 to repeat infinitely
    pub num_loops: u32,

    /// Whether animation time codes are present at animation frames in the
    /// codestream
    pub have_timecodes: JxlBool,
}

/// Basic image information. This information is available from the file
/// signature and first part of the codestream header.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlBasicInfo {
    /// Whether the codestream is embedded in the container format. If true,
    /// metadata information and extensions may be available in addition to the
    /// codestream.
    pub have_container: JxlBool,

    /// Width of the image in pixels, before applying orientation.
    pub xsize: u32,

    /// Height of the image in pixels, before applying orientation.
    pub ysize: u32,

    /// Original image color channel bit depth.
    pub bits_per_sample: u32,

    /// Original image color channel floating point exponent bits, or 0 if they
    /// are unsigned integer. For example, if the original data is half-precision
    /// (binary16) floating point, `bits_per_sample` is 16 and
    /// `exponent_bits_per_sample` is 5, and so on for other floating point
    /// precisions.
    pub exponent_bits_per_sample: u32,

    /// Upper bound on the intensity level present in the image in nits. For
    /// unsigned integer pixel encodings, this is the brightness of the largest
    /// representable value. The image does not necessarily contain a pixel
    /// actually this bright. An encoder is allowed to set 255 for SDR images
    /// without computing a histogram.
    /// Leaving this set to its default of 0 lets libjxl choose a sensible default
    /// value based on the color encoding.
    pub intensity_target: f32,

    /// Lower bound on the intensity level present in the image. This may be
    /// loose, i.e. lower than the actual darkest pixel. When tone mapping, a
    /// decoder will map `[min_nits, intensity_target]` to the display range.
    pub min_nits: f32,

    /// See the description of [`Self::linear_below`].
    pub relative_to_max_display: JxlBool,

    /// The tone mapping will leave unchanged (linear mapping) any pixels whose
    /// brightness is strictly below this. The interpretation depends on
    /// `relative_to_max_display`. If true, this is a ratio \[0, 1\] of the maximum
    /// display brightness \[nits\], otherwise an absolute brightness \[nits\].
    pub linear_below: f32,

    /// Whether the data in the codestream is encoded in the original color
    /// profile that is attached to the codestream metadata header, or is
    /// encoded in an internally supported absolute color space (which the decoder
    /// can always convert to linear or non-linear sRGB or to XYB). If the original
    /// profile is used, the decoder outputs pixel data in the color space matching
    /// that profile, but doesn't convert it to any other color space. If the
    /// original profile is not used, the decoder only outputs the data as sRGB
    /// (linear if outputting to floating point, nonlinear with standard sRGB
    /// transfer function if outputting to unsigned integers) but will not convert
    /// it to to the original color profile. The decoder also does not convert to
    /// the target display color profile. To convert the pixel data produced by
    /// the decoder to the original color profile, one of the `JxlDecoderGetColor*`
    /// functions needs to be called with
    /// [`JxlColorProfileTarget::Data`] to get the color profile of the decoder
    /// output, and then an external CMS can be used for conversion. Note that for
    /// lossy compression, this should be set to false for most use cases, and if
    /// needed, the image should be converted to the original color profile after
    /// decoding, as described above.
    pub uses_original_profile: JxlBool,

    /// Indicates a preview image exists near the beginning of the codestream.
    /// The preview itself or its dimensions are not included in the basic info.
    pub have_preview: JxlBool,

    /// Indicates animation frames exist in the codestream. The animation
    /// information is not included in the basic info.
    pub have_animation: JxlBool,

    /// Image orientation, value 1-8 matching the values used by JEITA CP-3451C
    /// (Exif version 2.3).
    pub orientation: JxlOrientation,

    /// Number of color channels encoded in the image, this is either 1 for
    /// grayscale data, or 3 for colored data. This count does not include
    /// the alpha channel or other extra channels. To check presence of an alpha
    /// channel, such as in the case of RGBA color, check `alpha_bits != 0`.
    /// If and only if this is `1`, the [`JxlColorSpace`] in the [`JxlColorEncoding`] is
    /// [`JxlColorSpace::Gray`].
    pub num_color_channels: u32,

    /// Number of additional image channels. This includes the main alpha channel,
    /// but can also include additional channels such as depth, additional alpha
    /// channels, spot colors, and so on. Information about the extra channels
    /// can be queried with [`JxlDecoderGetExtraChannelInfo`]. The main alpha
    /// channel, if it exists, also has its information available in the
    /// `alpha_bits`, `alpha_exponent_bits` and `alpha_premultiplied` fields in this
    /// [`JxlBasicInfo`].
    pub num_extra_channels: u32,

    /// Bit depth of the encoded alpha channel, or 0 if there is no alpha channel.
    /// If present, matches the `alpha_bits` value of the [`JxlExtraChannelInfo`]
    /// associated with this alpha channel.
    pub alpha_bits: u32,

    /// Alpha channel floating point exponent bits, or 0 if they are unsigned. If
    /// present, matches the `alpha_bits` value of the [`JxlExtraChannelInfo`] associated
    /// with this alpha channel. integer.
    pub alpha_exponent_bits: u32,

    /// Whether the alpha channel is premultiplied. Only used if there is a main
    /// alpha channel. Matches the `alpha_premultiplied` value of the
    /// [`JxlExtraChannelInfo`] associated with this alpha channel.
    pub alpha_premultiplied: JxlBool,

    /// Dimensions of encoded preview image, only used if `have_preview` is
    /// [`JxlBool::True`].
    pub preview: JxlPreviewHeader,

    /// Animation header with global animation properties for all frames, only
    /// used if `have_animation` is [`JxlBool::True`].
    pub animation: JxlAnimationHeader,

    /// Intrinsic width of the image.
    /// The intrinsic size can be different from the actual size in pixels
    /// (as given by xsize and ysize) and it denotes the recommended dimensions
    /// for displaying the image, i.e. applications are advised to resample the
    /// decoded image to the intrinsic dimensions.
    pub intrinsic_xsize: u32,

    /// Intrinsic height of the image.
    /// The intrinsic size can be different from the actual size in pixels
    /// (as given by xsize and ysize) and it denotes the recommended dimensions
    /// for displaying the image, i.e. applications are advised to resample the
    /// decoded image to the intrinsic dimensions.
    pub intrinsic_ysize: u32,

    /// Padding for forwards-compatibility, in case more fields are exposed
    /// in a future version of the library.
    _padding: [u8; 100],
}

/// Information for a single extra channel.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlExtraChannelInfo {
    /// Given type of an extra channel.
    pub r#type: JxlExtraChannelType,

    /// Total bits per sample for this channel.
    pub bits_per_sample: u32,

    /// Floating point exponent bits per channel, or 0 if they are unsigned
    /// integer.
    pub exponent_bits_per_sample: u32,

    /// The exponent the channel is downsampled by on each axis.
    /// TODO(lode): expand this comment to match the JPEG XL specification,
    /// specify how to upscale, how to round the size computation, and to which
    /// extra channels this field applies.
    pub dim_shift: u32,

    /// Length of the extra channel name in bytes, or 0 if no name.
    /// Excludes null termination character.
    pub name_length: u32,

    /// Whether alpha channel uses premultiplied alpha. Only applicable if
    /// type is [`JxlExtraChannelType::Alpha`].
    pub alpha_premultiplied: JxlBool,

    /// Spot color of the current spot channel in linear RGBA. Only applicable if
    /// type is [`JxlExtraChannelType::SpotColor`].
    pub spot_color: [f32; 4],

    /// Only applicable if type is [`JxlExtraChannelType::Cfa`].
    /// TODO(lode): add comment about the meaning of this field.
    pub cfa_channel: u32,
}
/// Extensions in the codestream header.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlHeaderExtensions {
    /// Extension bits.
    pub extensions: u64,
}
/// Frame blend modes.
/// When decoding, if coalescing is enabled (default), this can be ignored.
#[repr(C)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum JxlBlendMode {
    Replace = 0,
    Add = 1,
    Blend = 2,
    MULADD = 3,
    MUL = 4,
}

/// The information about blending the color channels or a single extra channel.
/// When decoding, if coalescing is enabled (default), this can be ignored and
/// the blend mode is considered to be [`JxlBlendMode::Replace`].
/// When encoding, these settings apply to the pixel data given to the encoder.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlBlendInfo {
    /// Blend mode.
    pub blendmode: JxlBlendMode,
    /// Reference frame ID to use as the 'bottom' layer (0-3).
    pub source: u32,
    /// Which extra channel to use as the 'alpha' channel for blend modes
    /// [`JxlBlendMode::Blend`] and [`JxlBlendMode::MULADD`].
    pub alpha: u32,
    /// Clamp values to \[0,1\] for the purpose of blending.
    pub clamp: JxlBool,
}

/// The information about layers.
/// When decoding, if coalescing is enabled (default), this can be ignored.
/// When encoding, these settings apply to the pixel data given to the encoder,
/// the encoder could choose an internal representation that differs.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlLayerInfo {
    /// Whether cropping is applied for this frame. When decoding, if false,
    /// [`Self::crop_x0`] and [`Self::crop_y0`] are set to zero, and [`Self::xsize`] and [`Self::ysize`] to the main
    /// image dimensions. When encoding and this is false, those fields are
    /// ignored. When decoding, if coalescing is enabled (default), this is always
    /// false, regardless of the internal encoding in the JPEG XL codestream.
    pub have_crop: JxlBool,

    /// Horizontal offset of the frame (can be negative).
    pub crop_x0: i32,

    /// Vertical offset of the frame (can be negative).
    pub crop_y0: i32,

    /// Width of the frame (number of columns).
    pub xsize: u32,

    /// Height of the frame (number of rows).
    pub ysize: u32,

    /// The blending info for the color channels. Blending info for extra channels
    /// has to be retrieved separately using [`JxlDecoderGetExtraChannelBlendInfo`].
    pub blend_info: JxlBlendInfo,

    /// After blending, save the frame as reference frame with this ID (0-3).
    /// Special case: if the frame duration is nonzero, ID 0 means "will not be
    /// referenced in the future". This value is not used for the last frame.
    /// When encoding, ID 3 is reserved to frames that are generated internally by
    /// the encoder, and should not be used by applications.
    pub save_as_reference: u32,
}

///The header of one displayed frame or non-coalesced layer.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlFrameHeader {
    /// How long to wait after rendering in ticks. The duration in seconds of a
    /// tick is given by [`JxlAnimationHeader::tps_numerator`] and [`JxlAnimationHeader::tps_denominator`] in
    /// [`JxlAnimationHeader`].
    pub duration: u32,

    /// SMPTE timecode of the current frame in form 0xHHMMSSFF, or 0. The bits are
    /// interpreted from most-significant to least-significant as hour, minute,
    /// second, and frame. If timecode is nonzero, it is strictly larger than that
    /// of a previous frame with nonzero duration. These values are only available
    /// if `have_timecodes` in [`JxlAnimationHeader`] is [`JxlBool::True`].
    /// This value is only used if `have_timecodes` in [`JxlAnimationHeader`] is
    /// [`JxlBool::True`].
    pub timecode: u32,

    /// Length of the frame name in bytes, or 0 if no name.
    /// Excludes null termination character. This value is set by the decoder.
    /// For the encoder, this value is ignored and [`JxlEncoderSetFrameName`] is
    /// used instead to set the name and the length.
    pub name_length: u32,

    /** Indicates this is the last animation frame. This value is set by the
     * decoder to indicate no further frames follow. For the encoder, it is not
     * required to set this value and it is ignored, [`JxlEncoderCloseFrames`] is
     * used to indicate the last frame to the encoder instead.
     */
    pub is_last: JxlBool,
    /** Information about the layer in case of no coalescing.
     */
    pub layer_info: JxlLayerInfo,
}
