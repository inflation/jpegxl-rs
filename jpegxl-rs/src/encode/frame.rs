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

use jpegxl_sys::common::types::{JxlEndianness, JxlPixelFormat};

use crate::common::PixelType;

use super::FrameSettings;

/// A frame for the encoder, consisting of the pixels and its options
#[allow(clippy::module_name_repetitions)]
pub struct EncoderFrame<'data, T: PixelType> {
    pub(crate) data: &'data [T],
    num_channels: Option<u32>,
    endianness: Option<JxlEndianness>,
    align: Option<usize>,
    pub(crate) settings: Option<&'data FrameSettings>,
    pub(crate) duration: Option<u32>,
    pub(crate) name: Option<&'data str>,
    pub(crate) bit_depth_from_image: bool,
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
            duration: None,
            name: None,
            bit_depth_from_image: false,
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

    /// Set how long the frame is shown, in ticks of [`Animation`](super::Animation).
    /// Ignored if the image has no animation
    #[must_use]
    pub fn duration(mut self, ticks: u32) -> Self {
        self.duration = Some(ticks);
        self
    }

    /// Set the name of the frame
    #[must_use]
    pub fn name(mut self, value: &'data str) -> Self {
        self.name = Some(value);
        self
    }

    /// The samples use the bit depth of the image instead of the full range of `T`,
    /// e.g. `0..=1023` in `u16` for a 10-bit image
    #[must_use]
    pub fn bit_depth_from_image(mut self) -> Self {
        self.bit_depth_from_image = true;
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
