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

use std::marker::PhantomData;

use jpegxl_sys::common::types::{JxlEndianness, JxlPixelFormat};

use crate::{common::PixelType, EncodeError};

use super::{EncoderResult, FrameSettings, Session};

/// A frame for the encoder, consisting of the pixels and its options
#[allow(clippy::module_name_repetitions)]
pub struct EncoderFrame<'data, T: PixelType> {
    pub(crate) data: &'data [T],
    num_channels: Option<u32>,
    endianness: Option<JxlEndianness>,
    align: Option<usize>,
    pub(crate) settings: Option<&'data FrameSettings>,
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
            settings: None,
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

    /// Use these settings instead of the encoder defaults
    #[must_use]
    pub fn settings(mut self, value: &'data FrameSettings) -> Self {
        self.settings = Some(value);
        self
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
    pub(crate) Session<'enc, 'prl, 'mm>,
    pub(crate) PhantomData<U>,
);

impl<U: PixelType> MultiFrames<'_, '_, '_, U> {
    /// Add a frame to the encoder
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to add a frame
    pub fn add_frame<T: PixelType>(mut self, frame: &EncoderFrame<T>) -> Result<Self, EncodeError> {
        self.0.add_frame(frame)?;
        Ok(self)
    }

    /// Add a JPEG raw frame to the encoder
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to add a jpeg frame
    pub fn add_jpeg_frame(mut self, data: &[u8]) -> Result<Self, EncodeError> {
        self.0.add_jpeg_frame(data)?;
        Ok(self)
    }

    /// Encode a JPEG XL image from the frames
    /// # Errors
    /// Return [`EncodeError`] if the internal encoder fails to encode
    pub fn encode(self) -> Result<EncoderResult<U>, EncodeError> {
        Ok(EncoderResult {
            data: self.0.finish()?,
            _pixel_type: PhantomData,
        })
    }
}
