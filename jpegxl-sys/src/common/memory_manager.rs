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

//! Abstraction functions used by JPEG XL to allocate memory.

use std::ffi::c_void;

/// Allocating function for a memory region of a given size.
///
/// Allocates a contiguous memory region of size `size` bytes. The returned
/// memory may not be aligned to a specific size or initialized at all.
///
/// # Parameters
/// - `opaque`: custom memory manager handle provided by the caller.
/// - `size`: size in bytes of the requested memory region.
///
/// # Returns
/// - `NULL` if the memory cannot be allocated.
/// - Pointer to the memory otherwise.
pub type JpegxlAllocFunc =
    unsafe extern "C-unwind" fn(opaque: *mut c_void, size: usize) -> *mut c_void;

/// Deallocating function pointer type.
///
/// This function **MUST** do nothing if `address` is `NULL`.
///
/// # Parameters
/// - `opaque`: custom memory manager handle provided by the caller.
/// - `address`: memory region pointer returned by [`JpegxlAllocFunc`], or `NULL`.
pub type JpegxlFreeFunc = unsafe extern "C-unwind" fn(opaque: *mut c_void, address: *mut c_void);

/// Memory Manager struct.
/// These functions, when provided by the caller, will be used to handle memory
/// allocations.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct JxlMemoryManager {
    /// The opaque pointer that will be passed as the first parameter to all the
    /// functions in this struct.
    pub opaque: *mut c_void,

    /// Memory allocation function. This can be NULL if and only if also the
    /// [`Self::free`] member in this class is NULL. All dynamic memory will be allocated
    /// and freed with these functions if they are not NULL, otherwise with the
    /// standard malloc/free.
    pub alloc: Option<JpegxlAllocFunc>,

    /// Free function matching the [`Self::alloc`] member.
    pub free: Option<JpegxlFreeFunc>,
}
