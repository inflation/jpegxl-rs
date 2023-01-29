use byteorder::{ByteOrder, NativeEndian, BE, LE};
use half::f16;
use jpegxl_sys::{JxlDataType, JxlPixelFormat};

use crate::Endianness;

use super::Orientation;

/// Result of decoding
#[derive(Debug)]
pub struct Metadata {
    /// Width of the image
    pub width: u32,
    /// Height of the image
    pub height: u32,
    /// Upper bound on the intensity level present in the image in nits
    pub intensity_target: f32,
    /// Lower bound on the intensity level present in the image
    pub min_nits: f32,
    /// Orientation
    pub orientation: Orientation,
    /// Number of color channels per pixel _without_ alpha channel, from metadata
    pub num_color_channels: u32,
    /// Whether the image has an alpha channel, from metadata
    pub has_alpha_channel: bool,
    /// Intrinsic width of the image.
    /// Applications are advised to resample the decoded image to the intrinsic dimensions
    pub intrinsic_width: u32,
    /// Intrinsic height of the image.
    /// Applications are advised to resample the decoded image to the intrinsic dimensions
    pub intrinsic_height: u32,
    /// ICC profile
    pub icc_profile: Option<Vec<u8>>,
}

/// Pixels returned from the decoder
pub enum Data {
    /// `f32` pixels
    Float(Vec<f32>),
    /// `u8` pixels
    Uint8(Vec<u8>),
    /// `u16` pixels
    Uint16(Vec<u16>),
    /// `f16` pixels
    Float16(Vec<f16>),
}

impl Data {
    pub(crate) fn new(data: Vec<u8>, pixel_format: &JxlPixelFormat) -> Self {
        match pixel_format.data_type {
            JxlDataType::Float => Self::Float(to_f32(&data, pixel_format)),
            JxlDataType::Uint8 => Self::Uint8(data),
            JxlDataType::Uint16 => Self::Uint16(to_u16(&data, pixel_format)),
            JxlDataType::Float16 => Self::Float16(to_f16(&data, pixel_format)),
        }
    }
}

pub(crate) fn to_f32(data: &[u8], pixel_format: &JxlPixelFormat) -> Vec<f32> {
    let mut buf = vec![f32::default(); data.len() / std::mem::size_of::<f32>()];
    match pixel_format.endianness {
        Endianness::Native => NativeEndian::read_f32_into(data, buf.as_mut_slice()),
        Endianness::Little => LE::read_f32_into(data, buf.as_mut_slice()),
        Endianness::Big => BE::read_f32_into(data, buf.as_mut_slice()),
    }
    buf
}

pub(crate) fn to_u16(data: &[u8], pixel_format: &JxlPixelFormat) -> Vec<u16> {
    let mut buf = vec![u16::default(); data.len() / std::mem::size_of::<u16>()];
    match pixel_format.endianness {
        Endianness::Native => NativeEndian::read_u16_into(data, buf.as_mut_slice()),
        Endianness::Little => LE::read_u16_into(data, buf.as_mut_slice()),
        Endianness::Big => BE::read_u16_into(data, buf.as_mut_slice()),
    }
    buf
}

pub(crate) fn to_f16(data: &[u8], pixel_format: &JxlPixelFormat) -> Vec<f16> {
    data.chunks_exact(std::mem::size_of::<f16>())
        .map(|v| {
            f16::from_bits(match pixel_format.endianness {
                Endianness::Native => NativeEndian::read_u16(v),
                Endianness::Little => LE::read_u16(v),
                Endianness::Big => BE::read_u16(v),
            })
        })
        .collect()
}
