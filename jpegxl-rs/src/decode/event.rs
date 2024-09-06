use std::ffi::c_int;

/// Target of the color profile.
pub use jpegxl_sys::decode::JxlColorProfileTarget as ColorProfileTarget;

use super::{ColorEncodingConfig, Config};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    BasicInfo,
    ColorEncoding(ColorEncodingConfig),
}

impl From<Event> for c_int {
    fn from(value: Event) -> Self {
        match value {
            Event::BasicInfo => 0x40,
            Event::ColorEncoding { .. } => 0x100,
        }
    }
}

pub(crate) fn parse_events<I>(iter: I) -> (c_int, Config)
where
    I: IntoIterator<Item = Event>,
{
    iter.into_iter()
        .fold((0, Config::default()), |(flag, config), x| {
            let flag = flag | c_int::from(x);
            let config = match x {
                Event::ColorEncoding(val) => Config {
                    color_profile: Some(val),
                },
                _ => config,
            };
            (flag, config)
        })
}
