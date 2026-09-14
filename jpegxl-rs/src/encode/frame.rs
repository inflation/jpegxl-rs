use std::marker::PhantomData;
use std::mem::MaybeUninit;

use jpegxl_sys::{
    common::types::{JxlEndianness, JxlPixelFormat},
    encoder::encode::{JxlEncoderInitBlendInfo, JxlEncoderInitFrameHeader},
    metadata::codestream_header::{JxlBlendInfo, JxlBlendMode, JxlFrameHeader, JxlLayerInfo},
};

use crate::{common::PixelType, EncodeError};

use super::{EncoderResult, JxlEncoder};

/// Blend mode
pub type BlendMode = JxlBlendMode;

/// Blend Info
#[derive(Debug, Clone)]
pub struct BlendInfo(JxlBlendInfo);

impl BlendInfo {
    /// Create new blend info
    pub fn new(blendmode: BlendMode, source: u32, alpha: u32, clamp: bool) -> Self {
        let mut value = MaybeUninit::uninit();
        let mut blend_info = unsafe {
            JxlEncoderInitBlendInfo(value.as_mut_ptr());
            value.assume_init()
        };

        blend_info.blendmode = blendmode;
        blend_info.source = source;
        blend_info.alpha = alpha;
        blend_info.clamp = clamp.into();

        Self(blend_info)
    }
}

/// Layer Information
#[derive(Debug, Clone)]
pub struct LayerInfo(JxlLayerInfo);

impl LayerInfo {
    /// Create new layer info
    pub fn new(
        have_crop: bool,
        crop_x0: i32,
        crop_y0: i32,
        xsize: u32,
        ysize: u32,
        blend_info: BlendInfo,
        save_as_reference: u32,
    ) -> Self {
        LayerInfo(JxlLayerInfo {
            have_crop: have_crop.into(),
            crop_x0,
            crop_y0,
            xsize,
            ysize,
            blend_info: blend_info.0,
            save_as_reference,
        })
    }
}

/// Additional informations for a frame
#[derive(Debug, Clone)]
pub struct FrameHeader(JxlFrameHeader);

impl FrameHeader {
    /// Create a default frame header
    pub fn new() -> Self {
        let mut value = MaybeUninit::uninit();
        unsafe {
            JxlEncoderInitFrameHeader(value.as_mut_ptr());
            Self(value.assume_init())
        }
    }

    /// Set duration
    pub fn duration(mut self, duration: u32) -> Self {
        self.0.duration = duration;
        self
    }

    /// Set timecode
    pub fn timecode(mut self, timecode: u32) -> Self {
        self.0.timecode = timecode;
        self
    }

    /// Set layer_info
    pub fn layer_info(mut self, layer_info: LayerInfo) -> Self {
        self.0.layer_info = layer_info.0;
        self
    }
}

/// A frame for the encoder, consisting of the pixels and its options
#[allow(clippy::module_name_repetitions)]
pub struct EncoderFrame<'data, T: PixelType> {
    pub(crate) data: &'data [T],
    num_channels: Option<u32>,
    endianness: Option<JxlEndianness>,
    align: Option<usize>,
    header: Option<FrameHeader>,
}

impl<'data, T: PixelType> EncoderFrame<'data, T> {
    /// Create a default frame from the data.
    ///
    /// Use RGB(3) channels, native endianness and no alignment.
    pub fn new(data: &'data [T]) -> Self {
        Self {
            data,
            num_channels: None,
            endianness: None,
            align: None,
            header: None,
        }
    }

    /// Set the number of channels of the source.
    ///
    /// _Note_: If you want to use alpha channel, add here
    #[must_use]
    pub fn num_channels(mut self, value: u32) -> Self {
        self.num_channels = Some(value);
        self
    }

    /// Set the endianness of the source.
    #[must_use]
    pub fn endianness(mut self, value: JxlEndianness) -> Self {
        self.endianness = Some(value);
        self
    }

    /// Set the align of the source.
    /// Align scanlines to a multiple of align bytes, or 0 to require no alignment at all
    #[must_use]
    pub fn align(mut self, value: usize) -> Self {
        self.align = Some(value);
        self
    }

    /// Set the frame header
    #[must_use]
    pub fn header(mut self, value: FrameHeader) -> Self {
        self.header = Some(value);
        self
    }

    /// Set the frame header
    pub(crate) fn get_header(&self) -> Option<&JxlFrameHeader> {
        self.header.as_ref().map(|f| &f.0)
    }

    pub(crate) fn pixel_format(&self) -> JxlPixelFormat {
        JxlPixelFormat {
            num_channels: self.num_channels.unwrap_or(3),
            data_type: T::pixel_type(),
            endianness: self.endianness.unwrap_or(JxlEndianness::Native),
            align: self.align.unwrap_or(0),
        }
    }
}

/// A wrapper type for encoding multiple frames
pub struct MultiFrames<'enc, 'prl, 'mm, U>(
    pub(crate) &'enc mut JxlEncoder<'prl, 'mm>,
    pub(crate) PhantomData<U>,
)
where
    'prl: 'enc,
    'mm: 'enc;

impl<U: PixelType> MultiFrames<'_, '_, '_, U> {
    /// Add a frame to the encoder
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to add a frame
    pub fn add_frame<T: PixelType>(self, frame: &EncoderFrame<T>) -> Result<Self, EncodeError> {
        self.0.add_frame(frame)?;
        Ok(self)
    }

    /// Add a JPEG raw frame to the encoder
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to add a jpeg frame
    pub fn add_jpeg_frame(self, data: &[u8]) -> Result<Self, EncodeError> {
        self.0.add_jpeg_frame(data)?;
        Ok(self)
    }

    /// Encode a JPEG XL image from the frames
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to encode
    pub fn encode(self) -> Result<EncoderResult<U>, EncodeError> {
        self.0.start_encoding()
    }
}
