/*
 * This file is part of jpegxl-rs.
 *
 * jpegxl-rs is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * jpegxl-rs is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with jpegxl-rs.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::mem::MaybeUninit;

use jpegxl_sys::{
    color::color_encoding::JxlColorEncoding,
    encoder::encode::{self as api, JxlEncoderFrameSettingId},
};

/// Encoding speed
#[derive(Debug, Clone, Copy, Default)]
pub enum EncoderSpeed {
    /// Fastest, 1
    Lightning = 1,
    /// 2
    Thunder = 2,
    /// 3
    Falcon = 3,
    /// 4
    Cheetah,
    /// 5
    Hare,
    /// 6
    Wombat,
    /// 7, default
    #[default]
    Squirrel,
    /// 8
    Kitten,
    /// 9
    Tortoise,
    /// Slowest, 10
    Glacier,
}

/// Encoding color profile
#[derive(Debug, Clone)]
pub enum ColorEncoding {
    /// sRGB, default for int pixel types
    Srgb,
    /// Linear sRGB, default for float pixel types
    LinearSrgb,
    /// sRGB, images with only luma channel
    SrgbLuma,
    /// Linear sRGB with only luma channel
    LinearSrgbLuma,
    /// Custom
    Custom(JxlColorEncoding),
}

impl From<&ColorEncoding> for JxlColorEncoding {
    fn from(val: &ColorEncoding) -> Self {
        use ColorEncoding::{Custom, LinearSrgb, LinearSrgbLuma, Srgb, SrgbLuma};

        let mut color_encoding = MaybeUninit::uninit();

        // SAFETY: every arm that does not return initializes `color_encoding`
        unsafe {
            match val {
                Srgb => api::JxlColorEncodingSetToSRGB(color_encoding.as_mut_ptr(), false.into()),
                LinearSrgb => {
                    api::JxlColorEncodingSetToLinearSRGB(color_encoding.as_mut_ptr(), false.into());
                }
                SrgbLuma => {
                    api::JxlColorEncodingSetToSRGB(color_encoding.as_mut_ptr(), true.into());
                }
                LinearSrgbLuma => {
                    api::JxlColorEncodingSetToLinearSRGB(color_encoding.as_mut_ptr(), true.into());
                }
                Custom(e) => {
                    return e.clone();
                }
            }
            color_encoding.assume_init()
        }
    }
}

/// Settings applied to a frame.
/// [`JxlEncoder::frame_settings`](super::JxlEncoder::frame_settings) returns the encoder defaults
#[derive(Debug, Clone)]
pub struct FrameSettings {
    /// See [`JxlEncoder::lossless`](super::JxlEncoder::lossless)
    pub lossless: Option<bool>,
    /// See [`JxlEncoder::speed`](super::JxlEncoder::speed)
    pub speed: EncoderSpeed,
    /// See [`JxlEncoder::quality`](super::JxlEncoder::quality)
    pub quality: f32,
    /// See [`JxlEncoder::decoding_speed`](super::JxlEncoder::decoding_speed)
    pub decoding_speed: i64,
    /// Raw options, applied after the fields above
    pub options: Vec<(JxlEncoderFrameSettingId, i64)>,
}
