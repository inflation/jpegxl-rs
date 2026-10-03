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

use std::{
    ffi::c_int,
    ops::{BitOr, BitOrAssign},
};

/// Target of the color profile.
pub use jpegxl_sys::decode::JxlColorProfileTarget as ColorProfileTarget;
use jpegxl_sys::metadata::codestream_header::JxlFrameHeader;

use super::{BasicInfo, Image};

/// Set of events a [`Session`](super::Session) subscribes to.
///
/// [`Events::BASIC_INFO`] is always subscribed to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Events(c_int);

impl Events {
    /// Basic information such as image dimensions and extra channels.
    pub const BASIC_INFO: Self = Self(0x40);
    /// Color encoding or ICC profile of the image.
    pub const COLOR_ENCODING: Self = Self(0x100);
    /// Preview image, if the image has one.
    pub const PREVIEW_IMAGE: Self = Self(0x200);
    /// Beginning of a displayed frame.
    pub const FRAME: Self = Self(0x400);
    /// A frame is fully decoded.
    pub const FULL_IMAGE: Self = Self(0x1000);
    /// JPEG reconstruction data is available.
    pub const JPEG_RECONSTRUCTION: Self = Self(0x2000);
    /// Header of a container box.
    pub const BOX: Self = Self(0x4000);
    /// A progressive step is reached, see [`Session::flush_image`](super::Session::flush_image).
    pub const FRAME_PROGRESSION: Self = Self(0x8000);
    /// A box is fully decoded. Only emitted if a box buffer was set.
    pub const BOX_COMPLETE: Self = Self(0x10000);

    /// No events.
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Whether all events in `other` are in `self`.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub(crate) const fn bits(self) -> c_int {
        self.0
    }
}

impl BitOr for Events {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Events {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Header of a displayed frame
#[derive(Debug, Clone)]
pub struct FrameInfo {
    /// Animation duration, timecode, last-frame flag and layer information
    pub header: JxlFrameHeader,
    /// Name of the frame, empty if it has none
    pub name: String,
}

/// Header of a container box
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoxHeader {
    /// Four character type, e.g. `b"Exif"`, `b"xml "` or `b"jumb"`.
    /// Decompressed type if [`JxlDecoder::decompress`](super::JxlDecoder::decompress) is set
    pub box_type: [u8; 4],
    /// Size of the box as it appears in the container, including its header
    pub size_raw: u64,
    /// Size of the box contents, if known
    pub size_contents: Option<u64>,
}

/// What the decoder reports back from [`Session::process`](super::Session::process)
#[derive(Debug)]
#[non_exhaustive]
pub enum Event {
    /// More input is needed. Pass the next chunk to `process`, or call
    /// [`Session::close_input`](super::Session::close_input) if there is none
    NeedMoreInput,
    /// Basic information, at most once per image
    BasicInfo(BasicInfo),
    /// Color encoding is available, see
    /// [`Session::icc_profile`](super::Session::icc_profile) and
    /// [`Session::color_encoding`](super::Session::color_encoding)
    ColorEncoding,
    /// The decoder needs a buffer for the preview, see
    /// [`Session::alloc_preview_buffer`](super::Session::alloc_preview_buffer)
    NeedPreviewOutBuffer,
    /// Preview image, `None` if no buffer was set
    PreviewImage(Option<Image>),
    /// Beginning of a frame
    Frame(FrameInfo),
    /// The decoder needs a buffer for the frame, see
    /// [`Session::alloc_image_buffer`](super::Session::alloc_image_buffer).
    /// Occurs again for every frame
    NeedImageOutBuffer,
    /// A progressive step is reached. [`Session::flush_image`](super::Session::flush_image)
    /// renders the image decoded so far
    FrameProgression,
    /// A frame is fully decoded. `None` if no buffer was set
    FullImage(Option<Image>),
    /// JPEG reconstruction data is available. The session collects the bytes and
    /// returns them as [`Event::Jpeg`] instead of producing pixels
    JpegReconstruction,
    /// The reconstructed JPEG file
    Jpeg(Vec<u8>),
    /// Header of a box, see [`Session::set_box_buffer`](super::Session::set_box_buffer)
    Box(BoxHeader),
    /// Contents of the box that was announced by the last [`Event::Box`]
    BoxComplete(Vec<u8>),
    /// Decoding is finished
    Success,
}
