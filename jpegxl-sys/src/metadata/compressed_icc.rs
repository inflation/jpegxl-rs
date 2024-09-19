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

//! Utility functions to compress and decompress ICC streams.

use crate::common::{memory_manager, types::JxlBool};

extern "C" {
    /// Allocates a buffer using the memory manager, fills it with a compressed
    /// representation of an ICC profile, returns the result through `output_buffer`
    /// and indicates its size through `output_size`.
    ///
    /// The result must be freed using the memory manager once it is not of any more
    /// use.
    ///
    /// # Parameters
    ///
    /// - `memory_manager`: Pointer to a `JxlMemoryManager`.
    /// - `icc`: Pointer to a buffer containing the uncompressed ICC profile.
    /// - `icc_size`: Size of the buffer containing the ICC profile.
    /// - `compressed_icc`: Will be set to a pointer to the buffer containing the result.
    /// - `compressed_icc_size`: Will be set to the size of the buffer containing the result.
    ///
    /// # Returns
    ///
    /// Whether compressing the profile was successful.
    pub fn JxlICCProfileEncode(
        memory_manager: *const memory_manager::JxlMemoryManager,
        icc: *const u8,
        icc_size: usize,
        compressed_icc: *mut *mut u8,
        compressed_icc_size: *mut usize,
    ) -> JxlBool;

    /// Allocates a buffer using the memory manager, fills it with the decompressed
    /// version of the ICC profile in `compressed_icc`, returns the result through
    /// `icc` and indicates its size through `icc_size`.
    ///
    /// The result must be freed using the memory manager once it is no longer needed.
    ///
    /// # Parameters
    ///
    /// - `memory_manager`: Pointer to a `JxlMemoryManager`.
    /// - `compressed_icc`: Pointer to a buffer containing the compressed ICC profile.
    /// - `compressed_icc_size`: Size of the buffer containing the compressed ICC profile.
    /// - `icc`: Will be set to a pointer to the buffer containing the result.
    /// - `icc_size`: Will be set to the size of the buffer containing the result.
    ///
    /// # Returns
    ///
    /// Whether decompressing the profile was successful.
    pub fn JxlICCProfileDecode(
        memory_manager: *const memory_manager::JxlMemoryManager,
        compressed_icc: *const u8,
        compressed_icc_size: usize,
        icc: *mut *mut u8,
        icc_size: *mut usize,
    ) -> JxlBool;
}
