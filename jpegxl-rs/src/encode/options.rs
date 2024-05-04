use std::mem::MaybeUninit;

use jpegxl_sys::{color_encoding::JxlColorEncoding, encode as api};

/// Encoding speed
#[derive(Debug, Clone, Copy)]
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
    Squirrel,
    /// 8
    Kitten,
    /// 9
    Tortoise,
    /// Slowest, 10
    Glacier,
}

impl std::default::Default for EncoderSpeed {
    fn default() -> Self {
        Self::Squirrel
    }
}

/// Encoding color profile
#[derive(Debug, Clone, Copy)]
pub enum ColorEncoding {
    /// SRGB, default for uint pixel types
    Srgb,
    /// Linear SRGB, default for float pixel types
    LinearSrgb,
    /// SRGB, images with only luma channel
    SrgbLuma,
    /// Linear SRGB with only luma channel
    LinearSrgbLuma,
}

impl From<ColorEncoding> for JxlColorEncoding {
    fn from(val: ColorEncoding) -> Self {
        use ColorEncoding::{LinearSrgb, LinearSrgbLuma, Srgb, SrgbLuma};

        let mut color_encoding = MaybeUninit::uninit();

        unsafe {
            match val {
                Srgb => api::JxlColorEncodingSetToSRGB(color_encoding.as_mut_ptr(), false),
                LinearSrgb => {
                    api::JxlColorEncodingSetToLinearSRGB(color_encoding.as_mut_ptr(), false);
                }
                SrgbLuma => api::JxlColorEncodingSetToSRGB(color_encoding.as_mut_ptr(), true),
                LinearSrgbLuma => {
                    api::JxlColorEncodingSetToLinearSRGB(color_encoding.as_mut_ptr(), true);
                }
            }
            color_encoding.assume_init()
        }
    }
}
