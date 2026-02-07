use std::mem::MaybeUninit;

use jpegxl_sys::{color::color_encoding::JxlColorEncoding, encoder::encode as api};

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
