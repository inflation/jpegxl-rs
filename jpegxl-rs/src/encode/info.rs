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

use bon::Builder;

/// Image-wide information of a [`Session`](super::Session)
///
/// ```
/// # use jpegxl_rs::encode::ImageInfo;
/// let info = ImageInfo::builder().width(64).height(64).has_alpha(true).build();
/// ```
#[derive(Debug, Clone, Builder)]
pub struct ImageInfo {
    /// Width of the image in pixels
    pub(crate) width: u32,
    /// Height of the image in pixels
    pub(crate) height: u32,
    /// Bits per sample of the encoded image
    ///
    /// Default: 8
    #[builder(default = 8)]
    pub(crate) bits_per_sample: u32,
    /// Exponent bits per sample, 0 for integer samples
    ///
    /// Default: 0
    #[builder(default)]
    pub(crate) exponent_bits_per_sample: u32,
    /// Whether the image has an alpha channel, with the same bit depth as the color channels
    ///
    /// Default: `false`
    #[builder(default)]
    pub(crate) has_alpha: bool,
    /// Animation timing. Without it, the frames are layers of a still image
    pub(crate) animation: Option<Animation>,
}

/// Animation timing of an image
///
/// ```
/// # use jpegxl_rs::encode::Animation;
/// let ten_fps = Animation::builder().tps_numerator(10).build();
/// ```
#[derive(Debug, Clone, Copy, Builder)]
pub struct Animation {
    /// Numerator of the ticks per second
    pub(crate) tps_numerator: u32,
    /// Denominator of the ticks per second
    ///
    /// Default: 1
    #[builder(default = 1)]
    pub(crate) tps_denominator: u32,
    /// Number of loops, or 0 to repeat forever
    ///
    /// Default: 0
    #[builder(default)]
    pub(crate) num_loops: u32,
}
