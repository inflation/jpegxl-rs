/*
This file is part of jpegxl-sys.

jpegxl-sys is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

jpegxl-sys is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with jpegxl-sys.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Utility functions to manipulate jhgm (gain map) boxes.

use crate::{color::color_encoding::JxlColorEncoding, common::types::JxlBool};

/// Gain map bundle
///
/// This structure is used to serialize gain map data to and from an input
/// buffer. It holds pointers to sections within the buffer, and different parts
/// of the gain map data such as metadata, ICC profile data, and the gain map
/// itself.
///
/// The pointers in this structure do not take ownership of the memory they point
/// to. Instead, they reference specific locations within the provided buffer. It
/// is the caller's responsibility to ensure that the buffer remains valid and is
/// not deallocated as long as these pointers are in use. The structure should be
/// considered as providing a view into the buffer, not as an owner of the data.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlGainMapBundle {
    /// Version number of the gain map bundle.
    pub jhgm_version: u8,
    /// Size of the gain map metadata in bytes.
    pub gain_map_metadata_size: u16,
    /// Pointer to the gain map metadata, which is a binary
    /// blob following ISO 21496-1. This pointer references data within the input
    /// buffer.
    pub gain_map_metadata: *const u8,
    /// Indicates whether a color encoding is present.
    pub has_color_encoding: JxlBool,
    /// If `has_color_encoding` is true, this field contains the
    /// uncompressed color encoding data.
    pub color_encoding: JxlColorEncoding,
    /// Size of the alternative ICC profile in bytes (compressed
    /// size).
    pub alt_icc_size: u32,
    /// Pointer to the compressed ICC profile. This pointer references
    /// data within the input buffer.
    pub alt_icc: *const u8,
    /// Size of the gain map in bytes.
    pub gain_map_size: u32,
    /// Pointer to the gain map data, which is a JPEG XL naked
    /// codestream. This pointer references data within the input buffer.
    pub gain_map: *const u8,
}

extern "C" {
    /// Calculates the total size required to serialize the gain map bundle into a
    /// binary buffer. This function accounts for all the necessary space to
    /// serialize fields such as gain map metadata, color encoding, compressed ICC
    /// profile data, and the gain map itself.
    ///
    /// # Parameters
    /// - `map_bundle`: A reference to the [`JxlGainMapBundle`] containing all
    ///   necessary data to compute the size.
    /// - `bundle_size`: A mutable reference to a `usize` where the size in bytes
    ///   required to serialize the bundle will be stored.
    ///
    /// # Returns
    /// - A boolean indicating whether setting the size was successful.
    pub fn JxlGainMapGetBundleSize(
        map_bundle: *const JxlGainMapBundle,
        bundle_size: *mut usize,
    ) -> JxlBool;

    /// Serializes the gain map bundle into a preallocated buffer. The function
    /// ensures that all parts of the bundle such as metadata, color encoding,
    /// compressed ICC profile, and the gain map are correctly encoded into the
    /// buffer. First call [`JxlGainMapGetBundleSize`] to get the size needed for
    /// the buffer.
    ///
    /// # Parameters
    /// - `map_bundle`: A pointer to the [`JxlGainMapBundle`] to serialize.
    /// - `output_buffer`: A pointer to the buffer where the serialized data
    ///   will be written.
    /// - `output_buffer_size`: The size of the output buffer in bytes. Must be
    ///   large enough to hold the entire serialized data.
    /// - `bytes_written`: A mutable reference to a `usize` where the number of bytes
    ///   written to the output buffer will be stored.
    ///
    /// # Returns
    /// - A boolean indicating whether writing the bundle was successful.
    pub fn JxlGainMapWriteBundle(
        map_bundle: *const JxlGainMapBundle,
        output_buffer: *mut u8,
        output_buffer_size: usize,
        bytes_written: *mut usize,
    ) -> JxlBool;

    /// Deserializes a gain map bundle from a provided buffer and populates a
    /// [`JxlGainMapBundle`] structure with the data extracted. This function assumes
    /// the buffer contains a valid serialized gain map bundle. After successful
    /// execution, the [`JxlGainMapBundle`] structure will reference three different
    /// sections within the buffer:
    ///  - `gain_map_metadata`
    ///  - `alt_icc`
    ///  - `gain_map`
    ///
    /// These sections will be accompanied by their respective sizes. Users must
    /// ensure that the buffer remains valid as long as these pointers are in use.
    ///
    /// # Parameters
    /// - `map_bundle`: Pointer to a preallocated [`JxlGainMapBundle`] where
    ///   the deserialized data will be stored.
    /// - `input_buffer`: Pointer to the buffer containing the serialized gain
    ///   map bundle data.
    /// - `input_buffer_size`: The size of the input buffer in bytes.
    /// - `bytes_read`: The number of bytes read from the input buffer.
    ///
    /// # Returns
    /// - A boolean indicating whether reading the bundle was successful.
    pub fn JxlGainMapReadBundle(
        map_bundle: *mut JxlGainMapBundle,
        input_buffer: *const u8,
        input_buffer_size: usize,
        bytes_read: *mut usize,
    ) -> JxlBool;
}
