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

//! Color Encoding definitions used by JPEG XL.
//! All CIE units are for the standard 1931 2 degree observer.

#[cfg(doc)]
use crate::metadata::codestream_header::JxlBasicInfo;

/// Color space of the image data.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlColorSpace {
    /// Tristimulus RGB
    Rgb = 0,
    /// Luminance based, the primaries in [`JxlColorEncoding`] must be ignored.
    /// This value implies that [`JxlBasicInfo::num_color_channels`] is `1`, any
    /// other value implies `num_color_channels` is `3`.
    Gray,
    /// XYB (opsin) color space
    Xyb,
    /// None of the other table entries describe the color space appropriately
    Unknown,
}

/// Built-in white points for color encoding. When decoding, the numerical xy
/// white point value can be read from the [`JxlColorEncoding::white_point`]
/// field regardless of the enum value. When encoding, enum values except
/// [`JxlWhitePoint::Custom`] override the numerical fields. Some enum values
/// match a subset of CICP (Rec. ITU-T H.273 | ISO/IEC 23091-2:2019(E)), however
/// the white point and RGB primaries are separate enums here.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlWhitePoint {
    /// CIE Standard Illuminant D65: 0.3127, 0.3290
    D65 = 1,
    /// White point must be read from the [`JxlColorEncoding::white_point`] field,
    /// or as ICC profile. This enum value is not an exact match of the
    /// corresponding CICP value.
    Custom = 2,
    /// CIE Standard Illuminant E (equal-energy): 1/3, 1/3
    E = 10,
    /// DCI-P3 from SMPTE RP 431-2: 0.314, 0.351
    Dci = 11,
}

/// Built-in primaries for color encoding. When decoding, the primaries can be
/// read from the [`JxlColorEncoding::primaries_red_xy`], [`JxlColorEncoding::primaries_green_xy`],
/// and [`JxlColorEncoding::primaries_blue_xy`] fields regardless of the enum value. When encoding,
/// the enum values except [`JxlPrimaries::Custom`] override the numerical fields. Some enum values
/// match a subset of CICP (Rec. ITU-T H.273 | ISO/IEC 23091-2:2019(E)), however the white point
/// and RGB primaries are separate enums here.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlPrimaries {
    /// The CIE xy values of the red, green and blue primaries are: 0.639998686,
    /// 0.330010138; 0.300003784, 0.600003357; 0.150002046, 0.059997204
    SRgb = 1,
    /// Primaries must be read from the [`JxlColorEncoding::primaries_red_xy`],
    /// [`JxlColorEncoding::primaries_green_xy`] and [`JxlColorEncoding::primaries_blue_xy`] fields,
    /// or as ICC profile. This enum value is not an exact match of the corresponding CICP value.
    Custom = 2,
    /// As specified in Rec. ITU-R BT.2100-1
    Rec2100 = 9,
    /// As specified in SMPTE RP 431-2
    P3 = 11,
}

/// Built-in transfer functions for color encoding. Enum values match a subset
/// of CICP (Rec. ITU-T H.273 | ISO/IEC 23091-2:2019(E)) unless specified
/// otherwise.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlTransferFunction {
    /// As specified in ITU-R BT.709-6
    BT709 = 1,
    /// None of the other table entries describe the transfer function.
    Unknown = 2,
    /// The gamma exponent is 1
    Linear = 8,
    /// As specified in IEC 61966-2-1 sRGB
    SRGB = 13,
    /// As specified in SMPTE ST 2084
    PQ = 16,
    /// As specified in SMPTE ST 428-1
    DCI = 17,
    /// As specified in Rec. ITU-R BT.2100-1
    HLG = 18,
    /// Transfer function follows power law given by the gamma value in [`JxlColorEncoding`].
    /// Not a CICP value.
    Gamma = 65535,
}

/// Rendering intent for color encoding, as specified in ISO 15076-1:2010
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JxlRenderingIntent {
    /// vendor-specific
    Perceptual = 0,
    /// media-relative
    Relative,
    /// vendor-specific
    Saturation,
    /// ICC-absolute
    Absolute,
}

/// Color encoding of the image as structured information.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct JxlColorEncoding {
    /// Color space of the image data.
    pub color_space: JxlColorSpace,

    /// Built-in white point. If this value is [`JxlWhitePoint::Custom`], must
    /// use the numerical white point values from [`Self::white_point_xy`].
    pub white_point: JxlWhitePoint,

    /// Numerical whitepoint values in CIE xy space.
    pub white_point_xy: [f64; 2],

    /// Built-in RGB primaries. If this value is [`JxlPrimaries::Custom`], must
    /// use the numerical primaries values below. This field and the custom values
    /// below are unused and must be ignored if the color space is
    /// [`JxlColorSpace::Gray`] or [`JxlColorSpace::Xyb`].
    pub primaries: JxlPrimaries,

    /// Numerical red primary values in CIE xy space.
    pub primaries_red_xy: [f64; 2],

    /// Numerical green primary values in CIE xy space.
    pub primaries_green_xy: [f64; 2],

    /// Numerical blue primary values in CIE xy space.
    pub primaries_blue_xy: [f64; 2],

    /// Transfer function if `have_gamma` is 0.
    pub transfer_function: JxlTransferFunction,

    /// Gamma value used when [`Self::transfer_function`] is [`JxlTransferFunction::Gamma`].
    pub gamma: f64,

    /// Rendering intent defined for the color profile.
    pub rendering_intent: JxlRenderingIntent,
}
