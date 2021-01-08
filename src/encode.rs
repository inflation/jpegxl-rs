/*
This file is part of jpegxl-rs.

jpegxl-rs is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

jpegxl-rs is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with jpegxl-rs.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::ffi::c_void;
use std::ptr::null;

use jpegxl_sys::*;

use crate::{
    common::*,
    error::{get_error, JXLEncodeError},
    memory::*,
    parallel::*,
};

/// JPEG XL Encoder
pub struct JXLEncoder<T: PixelType> {
    /// Opaque pointer to the underlying encoder
    enc: *mut JxlEncoder,

    /// Pixel format
    pub pixel_format: JxlPixelFormat,
    _pixel_type: std::marker::PhantomData<T>,

    /// Memory Manager
    pub memory_manager: Option<Box<dyn JXLMemoryManager>>,

    /// Parallel Runner
    pub parallel_runner: Option<Box<dyn JXLParallelRunner>>,
}
