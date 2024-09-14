use std::ffi::c_int;

use jpegxl_sys::common::types::JxlPixelFormat;
/// Target of the color profile.
pub use jpegxl_sys::decode::JxlColorProfileTarget as ColorProfileTarget;

use super::{ColorEncodingConfig, Config};

/// Events that can be subscribed to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// Basic information.
    BasicInfo,
    /// Color encoding.
    ColorEncoding(ColorEncodingConfig),
    /// Preview image.
    PreviewImage {
        /// Pixel format.
        pixel_format: JxlPixelFormat,
    },
    /// JPEG reconstruction.
    JpegReconstruction {
        /// Initial buffer size. Increase it to reduce the number of reallocations.
        init_buffer_size: usize,
    },
}

impl From<Event> for c_int {
    fn from(value: Event) -> Self {
        match value {
            Event::BasicInfo => 0x40,
            Event::ColorEncoding(_) => 0x100,
            Event::PreviewImage { .. } => 0x200,
            Event::JpegReconstruction { .. } => 0x2000,
        }
    }
}

pub(crate) fn parse_events<I>(iter: I) -> (c_int, Config)
where
    I: IntoIterator<Item = Event>,
{
    iter.into_iter()
        .fold((0, Config::default()), |(flag, mut config), x| {
            let flag = flag | c_int::from(x);
            let config = match x {
                Event::ColorEncoding(val) => {
                    config.color_profile = Some(val);
                    config
                }
                _ => config,
            };
            (flag, config)
        })
}
