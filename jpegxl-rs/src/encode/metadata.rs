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

use jpegxl_sys::common::types::JxlBoxType;

/// Metadata box
pub enum Metadata<'d> {
    /// EXIF
    /// The contents of this box must be prepended by a 4-byte tiff header offset,
    /// which may be 4 zero bytes in case the tiff header follows immediately.
    Exif(&'d [u8]),
    /// XMP/IPTC metadata
    Xmp(&'d [u8]),
    /// JUMBF superbox
    Jumb(&'d [u8]),
    /// Custom Metadata.
    /// Type should not start with `jxl`, `JXL`, or conflict with other box type,
    /// and should be registered with MP4RA (mp4ra.org).
    Custom([u8; 4], &'d [u8]),
}

impl<'d> Metadata<'d> {
    pub(crate) fn parts(&self) -> ([u8; 4], &'d [u8]) {
        match *self {
            Metadata::Exif(data) => (*b"Exif", data),
            Metadata::Xmp(data) => (*b"xml ", data),
            Metadata::Jumb(data) => (*b"jumb", data),
            Metadata::Custom(t, data) => (t, data),
        }
    }

    #[must_use]
    pub(crate) fn box_type(t: [u8; 4]) -> JxlBoxType {
        // SAFETY: `u8` and `c_char` have the same layout
        JxlBoxType(unsafe { std::mem::transmute::<[u8; 4], [std::ffi::c_char; 4]>(t) })
    }
}
